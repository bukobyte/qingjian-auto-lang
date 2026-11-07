//! 应用语言规则、最近应用与访问状态；不依赖具体输入法框架或操作系统。

mod app;
mod config;
mod language;
mod recent;
mod rule;
mod session;
mod snapshot;

pub use app::AppInfo;
pub use config::AutoLanguageConfig;
pub use language::{InputLanguage, MACOS_LANGUAGES};
pub use recent::RecentApps;
pub use rule::LanguageRule;
pub use session::RuleSession;
pub use snapshot::{RuntimeSnapshot, SNAPSHOT_VERSION};
