//! 自动语言设置的状态、最近应用选择和视图。

mod action;
mod icons;
mod poller;
mod state;
mod target;
mod view;

pub use action::Action;
pub use state::AutoLanguage;
pub use target::Target;
pub use view::{menu, page, preview};

#[cfg(test)]
mod tests;
