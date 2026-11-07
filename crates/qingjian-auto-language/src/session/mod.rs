//! 普通应用访问与配置修订：同一访问只应用一次默认，尊重临时手动模式。

use crate::{AutoLanguageConfig, InputLanguage};

#[derive(Debug, Default)]
pub struct RuleSession {
    config: AutoLanguageConfig,

    active_app: Option<String>,

    pending: bool,
}

impl RuleSession {
    pub fn configure(&mut self, config: &AutoLanguageConfig) {
        let target_changed = self.active_app.as_deref().is_some_and(|app| {
            let old_rule = self
                .config
                .rules
                .iter()
                .find(|rule| rule.app_id.eq_ignore_ascii_case(app))
                .map(|rule| rule.language);
            let new_rule = config
                .rules
                .iter()
                .find(|rule| rule.app_id.eq_ignore_ascii_case(app))
                .map(|rule| rule.language);
            old_rule != new_rule || self.config.target(app) != config.target(app)
        });
        if target_changed || (!self.config.enabled && config.enabled) {
            self.pending = true;
        }
        self.config = config.clone();
    }

    /// 只传普通应用；设置和短暂系统面板由平台过滤，不结束当前访问。
    pub fn activate(&mut self, app_id: &str) -> bool {
        if app_id.trim().is_empty()
            || self
                .active_app
                .as_deref()
                .is_some_and(|old| old.eq_ignore_ascii_case(app_id))
        {
            return false;
        }
        self.active_app = Some(app_id.to_owned());
        self.pending = true;
        true
    }

    /// 仅在输入目标已确认且组合状态安全时调用，取出后由 Host 设置实际模式。
    pub fn decision(&mut self, app_id: &str, available: &[InputLanguage]) -> Option<InputLanguage> {
        if !self.config.enabled
            || !self.pending
            || !self
                .active_app
                .as_deref()
                .is_some_and(|active| active.eq_ignore_ascii_case(app_id))
        {
            return None;
        }
        let language = self.config.target(app_id);
        if !available.contains(&language) {
            return None;
        }
        self.pending = false;
        Some(language)
    }

    pub fn note_manual(&mut self, app_id: &str) {
        if self
            .active_app
            .as_deref()
            .is_some_and(|active| active.eq_ignore_ascii_case(app_id))
        {
            self.pending = false;
        }
    }

    pub fn invalidate(&mut self) {
        self.pending = true;
    }

    pub fn current_app(&self) -> Option<&str> {
        self.active_app.as_deref()
    }
}

#[cfg(test)]
mod tests;
