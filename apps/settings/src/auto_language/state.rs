//! 自动语言页的临时状态；最近应用由输入法提供，规则以配置文件为准。

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use bamboo_slip::{Pixmap, Waker};
use qingjian_platform::{
    AppInfo, AutoLanguageConfig, InputLanguage, MACOS_LANGUAGES, RuntimeSnapshot,
};

use super::poller::Poller;
use super::{Target, icons};

#[derive(Default)]
pub struct AutoLanguage {
    pub snapshot: Option<RuntimeSnapshot>,

    pub unavailable: bool,

    pub picker_open: bool,

    pub selected: Option<AppInfo>,

    pub chosen: Option<InputLanguage>,

    pub hint: Option<(InputLanguage, Instant)>,

    pub icons: HashMap<String, Arc<Pixmap>>,

    poller: Option<Poller>,
}

impl AutoLanguage {
    pub fn start(&mut self, config_path: &Path, waker: Option<Waker>) {
        if self.poller.is_none()
            && let Some(waker) = waker
        {
            self.snapshot = None;
            self.selected = None;
            self.unavailable = false;
            self.poller = Some(Poller::start(
                qingjian_platform::auto_language::socket_path(config_path),
                waker,
            ));
        }
    }

    pub fn stop(&mut self) {
        self.poller = None;
        self.picker_open = false;
        self.hint = None;
    }

    pub fn finish(&mut self) -> bool {
        let expired = self.hint.is_some_and(|(_, until)| Instant::now() >= until);
        if expired {
            self.hint = None;
        }
        let Some(outcome) = self.poller.as_ref().and_then(Poller::finish) else {
            return expired;
        };
        match outcome {
            Ok(snapshot) => {
                let changed = self.snapshot.as_ref() != Some(&snapshot) || self.unavailable;
                if self
                    .snapshot
                    .as_ref()
                    .is_some_and(|old| old.session_id != snapshot.session_id)
                {
                    self.selected = None;
                }
                for app in &snapshot.recent_apps {
                    if !self.icons.contains_key(&app.app_id)
                        && let Some(icon) = icons::load(app)
                    {
                        self.icons.insert(app.app_id.clone(), icon);
                    }
                }
                self.snapshot = Some(snapshot);
                self.unavailable = false;
                changed || expired
            }
            Err(error) => {
                let changed = self.snapshot.take().is_some() || !self.unavailable;
                self.selected = None;
                self.unavailable = true;
                if changed {
                    tracing::debug!(%error, "Auto language status unavailable");
                }
                changed || expired
            }
        }
    }

    pub fn languages(&self) -> &[InputLanguage] {
        self.snapshot.as_ref().map_or(MACOS_LANGUAGES, |snapshot| {
            snapshot.available_languages.as_slice()
        })
    }

    pub fn current(&self) -> Option<&AppInfo> {
        self.snapshot.as_ref()?.current_app.as_ref()
    }

    pub fn value(&self, target: Target, config: &AutoLanguageConfig) -> Option<InputLanguage> {
        match target {
            Target::Default => Some(config.default_language),
            Target::Current => self
                .current()
                .and_then(|app| {
                    config
                        .rules
                        .iter()
                        .find(|rule| rule.app_id.eq_ignore_ascii_case(&app.app_id))
                })
                .and_then(|rule| rule.language),
            Target::Rule(index) => config.rules.get(index).and_then(|rule| rule.language),
            Target::Picker => self.chosen,
        }
    }

    pub fn preview(&mut self, language: InputLanguage, waker: Option<Waker>) {
        self.hint = Some((language, Instant::now() + Duration::from_millis(850)));
        if let Some(waker) = waker {
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(850));
                waker.wake();
            });
        }
    }
}
