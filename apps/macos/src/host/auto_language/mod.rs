//! 应用规则与 Host 输入状态的接线；只在确认的目标会话中执行模式决定。

mod state;

pub(super) use state::AutoLanguageState;

use qingjian_platform::{AutoLanguageConfig, InputLanguage, MACOS_LANGUAGES, RuntimeSnapshot};

use crate::auto_language::{
    frontmost_application, settings_active, take_activations, take_settings_visit,
};
use crate::host::Host;
use crate::imk::modifiers;

impl Host {
    pub(super) fn configure_auto_language(&mut self, config: &AutoLanguageConfig) {
        let mut effective = config.clone();
        effective.enabled &= !self.mode_switch.english_off();
        self.auto_language.rules.configure(&effective);
        self.auto_language.enabled = effective.enabled;
        self.publish_auto_language();
    }

    /// 通知只维护应用访问与状态，不在别的应用或设置窗口里切换输入模式。
    pub fn poll_auto_language(&mut self) {
        if take_settings_visit() || settings_active() {
            self.auto_language.needs_caps_rebase = true;
        }
        for app in take_activations() {
            if self.auto_language.rules.activate(&app.app_id) {
                self.auto_language.needs_caps_rebase = true;
                self.auto_language.confirmed_app = None;
                self.auto_language.confirmed_language = None;
            }
            self.auto_language.recent.record(app);
        }
        self.publish_auto_language();
    }

    /// IMK 激活及首个事件分派前调用；迟到的其他应用事件不能覆盖当前目标。
    pub fn prepare_auto_language(&mut self, app_id: Option<&str>) {
        self.poll_auto_language();
        if settings_active() {
            return;
        }
        let Some(app_id) = app_id else { return };
        if let Some(frontmost) = frontmost_application() {
            if !frontmost.app_id.eq_ignore_ascii_case(app_id) {
                return;
            }
            if self.auto_language.rules.activate(app_id) {
                self.auto_language.needs_caps_rebase = true;
                self.auto_language.confirmed_app = None;
                self.auto_language.confirmed_language = None;
            }
            self.auto_language.recent.record(frontmost);
        } else {
            return;
        }
        if self.auto_language.needs_caps_rebase {
            self.mode_switch.rebase_caps(modifiers::caps_lock_on());
            self.auto_language.needs_caps_rebase = false;
        }
        if !self.engine.composition().is_empty() || !self.vietnamese.is_empty() {
            return;
        }
        if let Some(language) = self.auto_language.rules.decision(app_id, MACOS_LANGUAGES) {
            let before = self.current_input_language();
            self.mode_switch
                .set_language(language, modifiers::caps_lock_on());
            self.last_osd_english = Some(before == InputLanguage::English);
            self.last_osd_vietnamese = Some(before == InputLanguage::Vietnamese);
            self.refresh_mode_ui();
        } else if self.mode_switch.sync_caps(modifiers::caps_lock_on()) {
            self.note_manual_language();
            self.refresh_mode_ui();
        }
        self.auto_language.confirmed_app = Some(app_id.to_owned());
        self.auto_language.confirmed_language = Some(self.current_input_language());
        self.publish_auto_language();
    }

    pub fn current_input_language(&self) -> InputLanguage {
        if self.vietnamese_mode() {
            InputLanguage::Vietnamese
        } else if self.english_mode() {
            InputLanguage::English
        } else {
            InputLanguage::Chinese
        }
    }

    pub(super) fn note_manual_language(&mut self) {
        if let Some(app) = self.engine.application() {
            self.auto_language.rules.note_manual(app);
            self.auto_language.confirmed_app = Some(app.to_owned());
            self.auto_language.confirmed_language = Some(self.current_input_language());
        }
        self.publish_auto_language();
    }

    fn publish_auto_language(&self) {
        let current_app = self.auto_language.recent.as_slice().first().cloned();
        let current_language = current_app.as_ref().and_then(|app| {
            self.auto_language
                .confirmed_app
                .as_deref()
                .filter(|confirmed| confirmed.eq_ignore_ascii_case(&app.app_id))
                .and(self.auto_language.confirmed_language)
        });
        if let Some(server) = &self.auto_language.server {
            server.publish(RuntimeSnapshot {
                version: 1,
                session_id: self.auto_language.session_id.clone(),
                recent_apps: self.auto_language.recent.as_slice().to_vec(),
                current_app,
                current_language,
                available_languages: MACOS_LANGUAGES.to_vec(),
                enabled: self.auto_language.enabled,
            });
        }
    }
}
