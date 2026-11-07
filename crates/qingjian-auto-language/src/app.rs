//! 最近使用应用的显示信息；不包含窗口标题或输入内容。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppInfo {
    pub app_id: String,

    pub display_name: String,

    pub bundle_path: Option<String>,
}
