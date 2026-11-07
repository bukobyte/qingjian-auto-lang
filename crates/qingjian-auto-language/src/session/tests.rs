//! 应用访问、手动覆盖、修订与未知目标时序。

use crate::{AutoLanguageConfig, InputLanguage, LanguageRule, MACOS_LANGUAGES, RuleSession};

fn config() -> AutoLanguageConfig {
    AutoLanguageConfig {
        enabled: true,
        rules: vec![LanguageRule {
            app_id: "terminal".into(),
            display_name: "Terminal".into(),
            language: Some(InputLanguage::English),
        }],
        ..AutoLanguageConfig::default()
    }
}

#[test]
fn applies_once_per_visit_and_reenters_even_without_intervening_input() {
    let mut session = RuleSession::default();
    session.configure(&config());
    assert!(session.activate("terminal"));
    assert_eq!(
        session.decision("terminal", MACOS_LANGUAGES),
        Some(InputLanguage::English)
    );
    session.note_manual("terminal");
    assert!(!session.activate("TERMINAL"));
    assert_eq!(session.decision("terminal", MACOS_LANGUAGES), None);
    session.activate("finder");
    session.activate("terminal");
    assert_eq!(
        session.decision("terminal", MACOS_LANGUAGES),
        Some(InputLanguage::English)
    );
}

#[test]
fn settings_ignored_and_only_effective_config_changes_override_manual() {
    let mut config = config();
    let mut session = RuleSession::default();
    session.configure(&config);
    session.activate("terminal");
    session.decision("terminal", MACOS_LANGUAGES);
    session.note_manual("terminal");
    // 设置往返不传 activate，不结束普通应用访问。
    config.show_switch_hint = false;
    session.configure(&config);
    assert_eq!(session.decision("terminal", MACOS_LANGUAGES), None);
    config.rules[0].language = Some(InputLanguage::Vietnamese);
    session.configure(&config);
    assert_eq!(
        session.decision("terminal", MACOS_LANGUAGES),
        Some(InputLanguage::Vietnamese)
    );
    assert_eq!(session.decision("terminal", MACOS_LANGUAGES), None);
}

#[test]
fn stale_unknown_or_unsupported_client_does_not_consume_pending() {
    let mut config = config();
    config.rules[0].language = Some(InputLanguage::Japanese);
    let mut session = RuleSession::default();
    session.configure(&config);
    session.activate("terminal");
    assert_eq!(session.decision("other", MACOS_LANGUAGES), None);
    assert_eq!(session.decision("terminal", MACOS_LANGUAGES), None);
    assert_eq!(
        session.decision("terminal", &[InputLanguage::Japanese]),
        Some(InputLanguage::Japanese)
    );
}

#[test]
fn disable_keeps_mode_and_enable_or_default_revision_reapplies() {
    let mut config = config();
    let mut session = RuleSession::default();
    session.configure(&config);
    session.activate("notes");
    assert_eq!(
        session.decision("notes", MACOS_LANGUAGES),
        Some(InputLanguage::Chinese)
    );
    config.enabled = false;
    session.configure(&config);
    assert_eq!(session.decision("notes", MACOS_LANGUAGES), None);
    config.default_language = InputLanguage::Vietnamese;
    session.configure(&config);
    config.enabled = true;
    session.configure(&config);
    assert_eq!(
        session.decision("notes", MACOS_LANGUAGES),
        Some(InputLanguage::Vietnamese)
    );
}

#[test]
fn explicit_rule_edit_reapplies_even_when_matching_global_default() {
    let mut config = config();
    let mut session = RuleSession::default();
    session.configure(&config);
    session.activate("notes");
    session.decision("notes", MACOS_LANGUAGES);
    session.note_manual("notes");
    config.rules.push(LanguageRule {
        app_id: "notes".into(),
        display_name: "Notes".into(),
        language: Some(InputLanguage::Chinese),
    });
    session.configure(&config);
    assert_eq!(
        session.decision("notes", MACOS_LANGUAGES),
        Some(InputLanguage::Chinese)
    );
}
