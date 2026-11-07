//! 本次运行内最近十个应用；重复应用移到最前，淘汰不会改规则。

use crate::AppInfo;

#[derive(Debug, Default)]
pub struct RecentApps {
    apps: Vec<AppInfo>,
}

impl RecentApps {
    pub fn record(&mut self, app: AppInfo) {
        if app.app_id.trim().is_empty() {
            return;
        }
        self.apps
            .retain(|old| !old.app_id.eq_ignore_ascii_case(&app.app_id));
        self.apps.insert(0, app);
        self.apps.truncate(10);
    }

    pub fn as_slice(&self) -> &[AppInfo] {
        &self.apps
    }
}

#[cfg(test)]
mod tests;
