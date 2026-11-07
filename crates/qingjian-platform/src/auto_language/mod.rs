//! 自动语言状态的本地只读通道；历史只留在输入法内存中。

use std::path::{Path, PathBuf};

use qingjian_auto_language::InputLanguage;

use crate::i18n::tr;

#[cfg(unix)]
mod client;
#[cfg(unix)]
mod server;

#[cfg(unix)]
pub use client::query_status;
#[cfg(unix)]
pub use server::StatusServer;

/// 独立私有目录，避免修改用户已有配置目录的访问权限。
pub fn socket_path(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("auto-language")
        .join("status.sock")
}

pub fn language_label(language: InputLanguage) -> &'static str {
    tr(match language {
        InputLanguage::Chinese => "auto_language.chinese",
        InputLanguage::English => "auto_language.english",
        InputLanguage::Vietnamese => "auto_language.vietnamese",
        InputLanguage::Japanese => "auto_language.japanese",
    })
}

pub fn language_hint(language: InputLanguage) -> &'static str {
    tr(match language {
        InputLanguage::Chinese => "auto_language.hint_chinese",
        InputLanguage::English => "auto_language.hint_english",
        InputLanguage::Vietnamese => "auto_language.hint_vietnamese",
        InputLanguage::Japanese => "auto_language.hint_japanese",
    })
}

#[cfg(not(unix))]
pub fn query_status(_path: &Path) -> std::io::Result<qingjian_auto_language::RuntimeSnapshot> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "automatic language status is unavailable on this platform",
    ))
}

#[cfg(unix)]
const MAX_STATUS_BYTES: u64 = 64 * 1024;

#[cfg(unix)]
const IO_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(200);

#[cfg(all(test, unix))]
mod tests;
