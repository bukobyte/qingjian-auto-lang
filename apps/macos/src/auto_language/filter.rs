//! 普通应用的过滤规则；系统输入面板与青简自身不进入最近列表。

pub(super) fn is_settings_application(app_id: Option<&str>, executable: Option<&str>) -> bool {
    app_id.is_some_and(|id| id.eq_ignore_ascii_case("app.qingjian.settings"))
        || executable == Some("qingjian-settings")
}

pub(super) fn is_rule_target(
    app_id: &str,
    regular: bool,
    bundle_path: Option<&str>,
    own_id: Option<&str>,
) -> bool {
    if !regular || app_id.trim().is_empty() {
        return false;
    }
    let id = app_id.to_ascii_lowercase();
    if id.starts_with("app.qingjian.")
        || own_id.is_some_and(|own| own.eq_ignore_ascii_case(app_id))
        || id.starts_with("com.apple.inputmethod.")
        || id.ends_with(".helper")
        || id.contains(".helper.")
        || id.contains(".helper(")
        || bundle_path.is_some_and(|path| {
            path.split('/')
                .any(|part| part.eq_ignore_ascii_case("Helpers"))
        })
    {
        return false;
    }
    ![
        "com.apple.textinputmenuagent",
        "com.apple.characterpaletteim",
        "com.apple.pressandhold",
        "com.apple.keyboardviewer",
        "com.apple.spotlight",
        "com.apple.campo",
        "com.apple.systemuiserver",
    ]
    .contains(&id.as_str())
}

#[cfg(test)]
mod tests {
    use super::{is_rule_target, is_settings_application};

    #[test]
    fn ordinary_apps_do_not_require_input_or_window_information() {
        for app_id in [
            "com.apple.finder",
            "com.apple.Terminal",
            "com.microsoft.VSCode",
            "com.googlecode.iterm2",
            "com.openai.chat",
        ] {
            assert!(is_rule_target(app_id, true, None, None));
        }
    }

    #[test]
    fn background_helpers_and_system_panels_are_transparent() {
        assert!(!is_rule_target("com.example.agent", false, None, None));
        assert!(!is_rule_target("", true, None, None));
        assert!(!is_rule_target("  ", true, None, None));
        for app_id in [
            "com.example.editor.helper",
            "com.example.editor.helper.renderer",
            "com.apple.CharacterPaletteIM",
            "com.apple.inputmethod.Kotoeri",
            "com.apple.Spotlight",
        ] {
            assert!(!is_rule_target(app_id, true, None, None));
        }
        assert!(!is_rule_target(
            "com.example.renderer",
            true,
            Some("/Applications/Editor.app/Contents/Frameworks/Helpers/Renderer.app"),
            None,
        ));
        assert!(is_rule_target("com.example.helper-tool", true, None, None));
    }

    #[test]
    fn qingjian_and_the_actual_process_bundle_are_not_targets() {
        for app_id in [
            "app.qingjian.inputmethod",
            "app.qingjian.settings",
            "app.qingjian.clipboard",
        ] {
            assert!(!is_rule_target(app_id, true, None, None));
        }
        assert!(!is_rule_target(
            "org.example.CustomInput",
            true,
            None,
            Some("org.example.custominput"),
        ));
    }

    #[test]
    fn settings_identity_supports_bundled_and_development_executables() {
        assert!(is_settings_application(Some("app.qingjian.settings"), None));
        assert!(is_settings_application(None, Some("qingjian-settings")));
        assert!(!is_settings_application(Some("com.example.Settings"), None));
        assert!(!is_settings_application(None, Some("other-settings")));
    }
}
