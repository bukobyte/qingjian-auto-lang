//! 将系统应用对象转成规则使用的稳定标识与展示信息。

use objc2_app_kit::{NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace};
use qingjian_platform::AppInfo;

use super::filter::{is_rule_target, is_settings_application};

/// 只返回当前普通应用；设置程序和临时系统面板不成为规则目标。
pub fn frontmost_application() -> Option<AppInfo> {
    let application = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    application_info(&application)
}

/// 设置窗口的激活应对应用访问透明，不能触发离开后重入。
pub fn settings_active() -> bool {
    let Some(application) = NSWorkspace::sharedWorkspace().frontmostApplication() else {
        return false;
    };
    is_settings(&application)
}

pub(super) fn is_settings(application: &NSRunningApplication) -> bool {
    let app_id = application.bundleIdentifier().map(|id| id.to_string());
    let executable = application
        .executableURL()
        .and_then(|url| url.lastPathComponent())
        .map(|name| name.to_string());
    is_settings_application(app_id.as_deref(), executable.as_deref())
}

pub(super) fn application_info(application: &NSRunningApplication) -> Option<AppInfo> {
    let app_id = application.bundleIdentifier()?.to_string();
    let bundle_path = application
        .bundleURL()
        .and_then(|url| url.path())
        .map(|path| path.to_string());
    let own_id = NSRunningApplication::currentApplication()
        .bundleIdentifier()
        .map(|id| id.to_string());
    if !is_rule_target(
        &app_id,
        application.activationPolicy() == NSApplicationActivationPolicy::Regular,
        bundle_path.as_deref(),
        own_id.as_deref(),
    ) {
        return None;
    }
    let display_name = application
        .localizedName()
        .map(|name| name.to_string())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| app_id.clone());
    Some(AppInfo {
        app_id,
        display_name,
        bundle_path,
    })
}
