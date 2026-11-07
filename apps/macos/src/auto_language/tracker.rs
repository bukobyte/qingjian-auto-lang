//! 监听 NSWorkspace 应用激活，按到达顺序缓存普通应用事件。

use std::cell::{Cell, RefCell};
use std::ptr::NonNull;

use block2::RcBlock;
use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{
    NSRunningApplication, NSWorkspace, NSWorkspaceApplicationKey,
    NSWorkspaceDidActivateApplicationNotification,
};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSObjectProtocol, NSOperationQueue};
use qingjian_platform::AppInfo;

use super::application::{application_info, frontmost_application, is_settings};

thread_local! {
    static ACTIVATIONS: RefCell<Vec<AppInfo>> = const { RefCell::new(Vec::new()) };

    static SETTINGS_VISITED: Cell<bool> = const { Cell::new(false) };
}

/// 按事件顺序取走激活记录；这里不做 MRU 或规则求值。
pub fn take_activations() -> Vec<AppInfo> {
    ACTIVATIONS.with(|pending| std::mem::take(&mut *pending.borrow_mut()))
}

/// 设置不构成普通访问，但其 Caps Lock 变化不能误算成原应用的手动切换。
pub fn take_settings_visit() -> bool {
    SETTINGS_VISITED.with(|visited| visited.replace(false))
}

/// observer 的生命周期随输入法进程，释放时解除订阅。
pub struct ApplicationTracker {
    center: Retained<NSNotificationCenter>,

    observer: Retained<ProtocolObject<dyn NSObjectProtocol>>,

    _mtm: MainThreadMarker,
}

impl ApplicationTracker {
    pub fn new(mtm: MainThreadMarker) -> Self {
        let workspace = NSWorkspace::sharedWorkspace();
        let center = workspace.notificationCenter();
        let block = RcBlock::new(|notification: NonNull<NSNotification>| {
            // 通知中心在主队列执行此 block，thread_local 与 Host 的消费线程一致。
            let notification = unsafe { notification.as_ref() };
            let Some(info) = notification.userInfo() else {
                return;
            };
            // AppKit 为应用激活通知提供 NSRunningApplication，不从可能已变化的 frontmost 回读。
            let app = unsafe { info.objectForKey(NSWorkspaceApplicationKey) }
                .and_then(|object| object.downcast::<NSRunningApplication>().ok());
            if let Some(app) = app {
                if is_settings(&app) {
                    SETTINGS_VISITED.with(|visited| visited.set(true));
                }
                if let Some(info) = application_info(&app) {
                    record(info);
                }
            }
            // 设置获焦时 IMK 定时器会停止；即使本次目标透明，也要发布刚才应用的状态。
            // Host 重入时 with 返回 None，普通应用事件仍留在队列中，待首键或下次通知消费。
            crate::host::with(|host| host.poll_auto_language());
        });
        // SAFETY: 观察 NSWorkspace 的专用通知；block 无非 Send 捕获，在主队列运行。
        let observer = unsafe {
            center.addObserverForName_object_queue_usingBlock(
                Some(NSWorkspaceDidActivateApplicationNotification),
                Some(&workspace),
                Some(&NSOperationQueue::mainQueue()),
                &block,
            )
        };
        if let Some(app) = frontmost_application() {
            record(app);
        }
        Self {
            center,
            observer,
            _mtm: mtm,
        }
    }
}

impl Drop for ApplicationTracker {
    fn drop(&mut self) {
        // SAFETY: token 来自这个通知中心，且保留到解除订阅之后。
        unsafe { self.center.removeObserver((*self.observer).as_ref()) };
    }
}

fn record(app: AppInfo) {
    ACTIVATIONS.with(|pending| pending.borrow_mut().push(app));
}
