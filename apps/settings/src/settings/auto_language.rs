//! 自动语言页的消息接线，配置保存按应用标识修改最新文件。

use qingjian_platform::{AppInfo, Config, LanguageRule};

use super::Settings;
use crate::auto_language::{Action, Target};
use crate::i18n::tr;

impl Settings {
    pub(super) fn auto_language_message(&mut self, action: Action) {
        self.menu = None;
        match action {
            Action::OpenPicker => {
                self.auto_language.picker_open = true;
                self.auto_language.selected = self.auto_language.current().cloned();
                self.auto_language.chosen = self
                    .auto_language
                    .selected
                    .as_ref()
                    .and_then(|app| {
                        self.config
                            .auto_language
                            .rules
                            .iter()
                            .find(|rule| rule.app_id.eq_ignore_ascii_case(&app.app_id))
                    })
                    .and_then(|rule| rule.language);
            }
            Action::ClosePicker => self.auto_language.picker_open = false,
            Action::SelectApp(app_id) => {
                self.auto_language.selected = self
                    .auto_language
                    .snapshot
                    .as_ref()
                    .and_then(|snapshot| {
                        snapshot
                            .recent_apps
                            .iter()
                            .find(|app| app.app_id.eq_ignore_ascii_case(&app_id))
                    })
                    .cloned();
                self.auto_language.chosen = self
                    .config
                    .auto_language
                    .rules
                    .iter()
                    .find(|rule| rule.app_id.eq_ignore_ascii_case(&app_id))
                    .and_then(|rule| rule.language);
            }
            Action::Choose(Target::Default, Some(language)) => {
                self.write("auto_language", "default_language", language.key())
            }
            Action::Choose(Target::Default, None) => {}
            Action::Choose(Target::Picker, language) => self.auto_language.chosen = language,
            Action::Choose(Target::Current, language) => {
                if let Some(app) = self.auto_language.current().cloned() {
                    self.save_auto_language(app, language);
                }
            }
            Action::Choose(Target::Rule(index), language) => {
                if let Some(rule) = self.config.auto_language.rules.get(index).cloned() {
                    self.save_auto_language(
                        AppInfo {
                            app_id: rule.app_id,
                            display_name: rule.display_name,
                            bundle_path: None,
                        },
                        language,
                    );
                }
            }
            Action::SavePicker => {
                if let Some(app) = self.auto_language.selected.clone()
                    && self.save_auto_language(app, self.auto_language.chosen)
                {
                    self.auto_language.picker_open = false;
                }
            }
            Action::Remove(app_id) => {
                match Config::remove_auto_language_rule(&self.path, &app_id) {
                    Ok(()) => self.reload(),
                    Err(error) => self.notice = Some(("auto-language", error)),
                }
            }
            Action::Preview => {
                if self.config.auto_language.show_switch_hint {
                    let language = self
                        .auto_language
                        .current()
                        .map_or(self.config.auto_language.default_language, |app| {
                            self.config.auto_language.target(&app.app_id)
                        });
                    self.auto_language.preview(language, self.waker.clone());
                }
            }
        }
    }

    fn save_auto_language(
        &mut self,
        app: AppInfo,
        language: Option<qingjian_platform::InputLanguage>,
    ) -> bool {
        if language.is_some_and(|language| !self.auto_language.languages().contains(&language)) {
            self.notice = Some((
                "auto-language",
                tr("auto_language.language_unavailable").to_owned(),
            ));
            return false;
        }
        let rule = LanguageRule {
            app_id: app.app_id,
            display_name: app.display_name,
            language,
        };
        match Config::set_auto_language_rule(&self.path, &rule) {
            Ok(()) => {
                self.reload();
                true
            }
            Err(error) => {
                self.notice = Some(("auto-language", error));
                false
            }
        }
    }
}
