//! 页面可见期间在后台读取只读状态，避免界面线程等待 socket。

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bamboo_slip::Waker;
use qingjian_platform::RuntimeSnapshot;

pub(super) struct Poller {
    stop: Arc<AtomicBool>,

    outcome: Arc<Mutex<Option<Result<RuntimeSnapshot, String>>>>,
}

impl Poller {
    pub fn start(path: PathBuf, waker: Waker) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let outcome = Arc::new(Mutex::new(None));
        let worker_stop = Arc::clone(&stop);
        let worker_outcome = Arc::clone(&outcome);
        std::thread::spawn(move || {
            while !worker_stop.load(Ordering::Relaxed) {
                let result = qingjian_platform::auto_language::query_status(&path)
                    .map_err(|error| error.to_string());
                if worker_stop.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok(mut slot) = worker_outcome.lock() {
                    *slot = Some(result);
                }
                waker.wake();
                std::thread::sleep(Duration::from_millis(500));
            }
        });
        Self { stop, outcome }
    }

    pub fn finish(&self) -> Option<Result<RuntimeSnapshot, String>> {
        self.outcome.lock().ok()?.take()
    }
}

impl Drop for Poller {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
