//! 最近应用与只读状态查询的进程级生命周期。

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use objc2::MainThreadMarker;
use qingjian_platform::auto_language::{StatusServer, socket_path};
use qingjian_platform::{InputLanguage, RecentApps, RuleSession};

use crate::auto_language::ApplicationTracker;

pub(crate) struct AutoLanguageState {
    pub rules: RuleSession,

    pub recent: RecentApps,

    pub server: Option<StatusServer>,

    pub session_id: String,

    pub confirmed_app: Option<String>,

    pub confirmed_language: Option<InputLanguage>,

    pub enabled: bool,

    pub needs_caps_rebase: bool,

    _tracker: ApplicationTracker,
}

impl AutoLanguageState {
    pub fn new(mtm: MainThreadMarker, config_path: Option<&Path>) -> Self {
        let server = config_path.and_then(|path| match StatusServer::start(&socket_path(path)) {
            Ok(server) => Some(server),
            Err(error) => {
                tracing::warn!(%error, "Auto language status endpoint unavailable");
                None
            }
        });
        Self {
            rules: RuleSession::default(),
            recent: RecentApps::default(),
            server,
            session_id: format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            confirmed_app: None,
            confirmed_language: None,
            enabled: false,
            needs_caps_rebase: false,
            _tracker: ApplicationTracker::new(mtm),
        }
    }
}
