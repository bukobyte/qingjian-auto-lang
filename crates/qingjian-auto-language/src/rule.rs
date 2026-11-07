//! 单个应用的默认输入语言；空目标表示跟随全局默认。

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::InputLanguage;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanguageRule {
    pub app_id: String,

    #[serde(default)]
    pub display_name: String,

    #[serde(
        default,
        serialize_with = "serialize_language",
        deserialize_with = "deserialize_language"
    )]
    pub language: Option<InputLanguage>,
}

fn serialize_language<S: Serializer>(
    value: &Option<InputLanguage>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(value.map_or("default", InputLanguage::key))
}

fn deserialize_language<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<InputLanguage>, D::Error> {
    let value = String::deserialize(deserializer)?;
    match value.as_str() {
        "default" => Ok(None),
        "chinese" => Ok(Some(InputLanguage::Chinese)),
        "english" => Ok(Some(InputLanguage::English)),
        "vietnamese" => Ok(Some(InputLanguage::Vietnamese)),
        "japanese" => Ok(Some(InputLanguage::Japanese)),
        _ => Err(serde::de::Error::custom(format!(
            "unknown input language: {value}"
        ))),
    }
}
