//! 输入法到设置程序的只读内存快照。

use serde::{Deserialize, Serialize};

use crate::{AppInfo, InputLanguage, MACOS_LANGUAGES};

pub const SNAPSHOT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    pub version: u32,

    pub session_id: String,

    pub recent_apps: Vec<AppInfo>,

    pub current_app: Option<AppInfo>,

    pub current_language: Option<InputLanguage>,

    pub available_languages: Vec<InputLanguage>,

    pub enabled: bool,
}

impl Default for RuntimeSnapshot {
    fn default() -> Self {
        Self {
            version: SNAPSHOT_VERSION,
            session_id: String::new(),
            recent_apps: Vec::new(),
            current_app: None,
            current_language: None,
            available_languages: MACOS_LANGUAGES.to_vec(),
            enabled: false,
        }
    }
}
