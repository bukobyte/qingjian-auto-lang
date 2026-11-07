//! 自动语言的应用激活入口；只记录普通应用，不读取窗口或输入内容。

mod application;
mod filter;
mod tracker;

pub use application::{frontmost_application, settings_active};
pub use tracker::{ApplicationTracker, take_activations, take_settings_visit};
