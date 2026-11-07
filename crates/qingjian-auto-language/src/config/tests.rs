//! 语言规则的文件格式、缺省与非法配置验证。

use crate::{AutoLanguageConfig, InputLanguage, LanguageRule};

#[test]
fn defaults_and_default_rule_round_trip() {
    let default: AutoLanguageConfig = toml::from_str("").unwrap();
    assert!(!default.enabled);
    assert!(default.show_switch_hint);
    assert_eq!(default.default_language, InputLanguage::Chinese);
    let source = "default_language = 'vietnamese'\n[[rules]]\napp_id = 'com.example.App'\nlanguage = 'default'\n";
    let config: AutoLanguageConfig = toml::from_str(source).unwrap();
    assert_eq!(config.target("COM.EXAMPLE.APP"), InputLanguage::Vietnamese);
    assert_eq!(config.target("other"), InputLanguage::Vietnamese);
    assert_eq!(config.rules[0].language, None);
    let encoded = toml::to_string(&config).unwrap();
    assert!(encoded.contains("language = \"default\""));
    assert_eq!(
        toml::from_str::<AutoLanguageConfig>(&encoded).unwrap(),
        config
    );
}

#[test]
fn invalid_identifiers_and_unknown_languages_are_rejected() {
    for source in [
        "[[rules]]\napp_id = ''\nlanguage = 'english'",
        "[[rules]]\napp_id = ' app '\nlanguage = 'english'",
        "[[rules]]\napp_id = 'app'\nlanguage = 'klingon'",
        "[[rules]]\napp_id = 'app'\nlanguage = 'english'\n[[rules]]\napp_id = 'APP'\nlanguage = 'chinese'",
    ] {
        assert!(
            toml::from_str::<AutoLanguageConfig>(source).is_err(),
            "{source}"
        );
    }
}

#[test]
fn explicit_rule_overrides_default_and_known_unavailable_is_retained() {
    let config = AutoLanguageConfig {
        rules: vec![LanguageRule {
            app_id: "app".into(),
            display_name: "App".into(),
            language: Some(InputLanguage::Japanese),
        }],
        ..AutoLanguageConfig::default()
    };
    assert_eq!(config.target("APP"), InputLanguage::Japanese);
    let encoded = toml::to_string(&config).unwrap();
    assert_eq!(
        toml::from_str::<AutoLanguageConfig>(&encoded).unwrap(),
        config
    );
}
