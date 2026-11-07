//! 自动语言设置的窄范围状态与配置测试。

use bamboo_slip::App;
use qingjian_platform::{AppInfo, AutoLanguageConfig, Config, InputLanguage, RuntimeSnapshot};

use super::{Action, AutoLanguage, Target};
use crate::message::Message;
use crate::settings::Settings;

fn app(id: &str) -> AppInfo {
    AppInfo {
        app_id: id.to_owned(),
        display_name: id.to_owned(),
        bundle_path: None,
    }
}

fn settings(name: &str) -> (Settings, std::path::PathBuf) {
    let directory = std::env::temp_dir().join(format!(
        "qingjian-auto-language-ui-{name}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("config.toml");
    std::fs::write(&path, "[general]\npage_size = 7\n").unwrap();
    let mut settings = Settings::load(path);
    let app = app("sample.editor");
    settings.auto_language.snapshot = Some(RuntimeSnapshot {
        current_app: Some(app.clone()),
        recent_apps: vec![app],
        session_id: "test-session".to_owned(),
        ..RuntimeSnapshot::default()
    });
    (settings, directory)
}

#[test]
fn auto_language_current_dropdown_adds_updates_and_follows_default() {
    let (mut settings, directory) = settings("save");
    settings.update(Message::AutoLanguage(Action::Choose(
        Target::Current,
        Some(InputLanguage::Vietnamese),
    )));
    assert_eq!(settings.config.auto_language.rules.len(), 1);
    assert_eq!(
        settings.config.auto_language.rules[0].language,
        Some(InputLanguage::Vietnamese)
    );
    settings.update(Message::AutoLanguage(Action::Choose(Target::Current, None)));
    assert_eq!(settings.config.auto_language.rules.len(), 1);
    assert_eq!(settings.config.auto_language.rules[0].language, None);
    assert_eq!(settings.config.general.page_size, 7);
    assert_eq!(
        settings
            .auto_language
            .snapshot
            .as_ref()
            .unwrap()
            .current_language,
        None
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn auto_language_picker_updates_existing_app_without_duplicate() {
    let (mut settings, directory) = settings("picker");
    settings.update(Message::AutoLanguage(Action::Choose(
        Target::Current,
        Some(InputLanguage::English),
    )));
    settings.update(Message::AutoLanguage(Action::OpenPicker));
    assert!(settings.auto_language.picker_open);
    assert_eq!(settings.auto_language.chosen, Some(InputLanguage::English));
    settings.update(Message::AutoLanguage(Action::Choose(
        Target::Picker,
        Some(InputLanguage::Vietnamese),
    )));
    settings.update(Message::AutoLanguage(Action::SavePicker));
    assert!(!settings.auto_language.picker_open);
    assert_eq!(settings.config.auto_language.rules.len(), 1);
    assert_eq!(
        settings.config.auto_language.rules[0].language,
        Some(InputLanguage::Vietnamese)
    );
    settings.update(Message::AutoLanguage(Action::Remove(
        "sample.editor".to_owned(),
    )));
    assert!(
        Config::load(&directory.join("config.toml"))
            .unwrap()
            .auto_language
            .rules
            .is_empty()
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn auto_language_unknown_app_does_not_create_rule_or_actual_state() {
    let (mut settings, directory) = settings("unknown");
    settings.auto_language.snapshot = None;
    settings.update(Message::AutoLanguage(Action::Choose(
        Target::Current,
        Some(InputLanguage::English),
    )));
    assert!(settings.config.auto_language.rules.is_empty());
    assert!(settings.auto_language.current().is_none());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn auto_language_catalog_is_platform_capability_and_never_gloss_languages() {
    let mut state = AutoLanguage::default();
    assert!(!state.languages().contains(&InputLanguage::Japanese));
    state.snapshot = Some(RuntimeSnapshot {
        available_languages: vec![InputLanguage::Chinese, InputLanguage::English],
        ..RuntimeSnapshot::default()
    });
    assert_eq!(
        state.languages(),
        &[InputLanguage::Chinese, InputLanguage::English]
    );
    assert_eq!(
        state.value(Target::Default, &AutoLanguageConfig::default()),
        Some(InputLanguage::Chinese)
    );
}

#[test]
fn auto_language_preview_does_not_change_rules_or_actual_mode() {
    let (mut settings, directory) = settings("preview");
    let config = settings.config.clone();
    settings.update(Message::AutoLanguage(Action::Preview));
    assert!(settings.auto_language.hint.is_some());
    assert_eq!(settings.config, config);
    assert_eq!(
        settings
            .auto_language
            .snapshot
            .as_ref()
            .unwrap()
            .current_language,
        None
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn auto_language_new_query_cycle_clears_old_target_before_first_response() {
    let mut state = AutoLanguage::default();
    state.snapshot = Some(RuntimeSnapshot {
        current_app: Some(app("old.app")),
        ..RuntimeSnapshot::default()
    });
    state.selected = Some(app("old.app"));
    let path = std::env::temp_dir().join("qingjian-auto-language-ui-no-server/config.toml");
    state.start(&path, Some(bamboo_slip::Waker::noop()));
    assert!(state.current().is_none());
    assert!(state.selected.is_none());
    state.snapshot = Some(RuntimeSnapshot {
        current_app: Some(app("new.app")),
        ..RuntimeSnapshot::default()
    });
    state.start(&path, Some(bamboo_slip::Waker::noop()));
    assert_eq!(state.current().unwrap().app_id, "new.app");
    state.stop();
}
