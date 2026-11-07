//! 实际输入模式的稳定标识，不与候选译文语言表混用。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputLanguage {
    #[default]
    Chinese,

    English,

    Vietnamese,

    /// 其他平台已有实现，macOS 尚未接入；读配置时保留这个已知值。
    Japanese,
}

pub const MACOS_LANGUAGES: &[InputLanguage] = &[
    InputLanguage::Chinese,
    InputLanguage::English,
    InputLanguage::Vietnamese,
];

impl InputLanguage {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Chinese => "chinese",
            Self::English => "english",
            Self::Vietnamese => "vietnamese",
            Self::Japanese => "japanese",
        }
    }
}
