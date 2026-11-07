//! 自动语言配置与规则校验。

use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize};

use crate::{InputLanguage, LanguageRule};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoLanguageConfig {
    pub enabled: bool,

    pub default_language: InputLanguage,

    pub show_switch_hint: bool,

    #[serde(deserialize_with = "deserialize_rules")]
    pub rules: Vec<LanguageRule>,
}

impl Default for AutoLanguageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_language: InputLanguage::Chinese,
            show_switch_hint: true,
            rules: Vec::new(),
        }
    }
}

impl AutoLanguageConfig {
    pub fn validate(&self) -> Result<(), String> {
        validate_rules(&self.rules)
    }

    pub fn target(&self, app_id: &str) -> InputLanguage {
        self.rules
            .iter()
            .find(|rule| rule.app_id.eq_ignore_ascii_case(app_id))
            .and_then(|rule| rule.language)
            .unwrap_or(self.default_language)
    }
}

fn validate_rules(rules: &[LanguageRule]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for rule in rules {
        if rule.app_id.trim().is_empty() || rule.app_id.trim() != rule.app_id {
            return Err(
                "application identifier must be nonempty and have no surrounding whitespace".into(),
            );
        }
        if !ids.insert(rule.app_id.to_ascii_lowercase()) {
            return Err(format!("duplicate application identifier: {}", rule.app_id));
        }
    }
    Ok(())
}

fn deserialize_rules<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<LanguageRule>, D::Error> {
    let rules = Vec::<LanguageRule>::deserialize(deserializer)?;
    validate_rules(&rules).map_err(serde::de::Error::custom)?;
    Ok(rules)
}

#[cfg(test)]
mod tests;
