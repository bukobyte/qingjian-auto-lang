//! 自动语言单规则保存不能覆盖其他页面或并发编辑后的配置。

use crate::{Config, InputLanguage, LanguageRule};

#[test]
fn auto_language_rule_updates_preserve_other_settings_and_validate() {
    let path =
        std::env::temp_dir().join(format!("qingjian-auto-config-{}.toml", std::process::id()));
    std::fs::write(&path, "[general]\n# keep this comment\npage_size = 7\n").unwrap();
    let mut rule = LanguageRule {
        app_id: "com.example.App".into(),
        display_name: "App".into(),
        language: Some(InputLanguage::Vietnamese),
    };
    Config::set_auto_language_rule(&path, &rule).unwrap();
    Config::set_bool(&path, "auto_language", "enabled", true).unwrap();
    Config::set_value(&path, "general", "page_size", 8).unwrap();
    rule.language = None;
    Config::set_auto_language_rule(&path, &rule).unwrap();
    let config = Config::load(&path).unwrap();
    assert!(config.auto_language.enabled);
    assert_eq!(config.auto_language.rules.len(), 1);
    assert_eq!(config.auto_language.rules[0].language, None);
    assert_eq!(config.general.page_size(), 8);
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .contains("# keep this comment")
    );
    let before = std::fs::read_to_string(&path).unwrap();
    rule.app_id = "".into();
    assert!(Config::set_auto_language_rule(&path, &rule).is_err());
    assert_eq!(before, std::fs::read_to_string(&path).unwrap());
    Config::remove_auto_language_rule(&path, "COM.EXAMPLE.APP").unwrap();
    assert!(Config::load(&path).unwrap().auto_language.rules.is_empty());
    std::fs::write(
        &path,
        "[[auto_language.rules]]\napp_id = 'app'\nlanguage = 'bad'\n",
    )
    .unwrap();
    assert!(Config::load(&path).is_err());
    std::fs::remove_file(path).unwrap();
}
