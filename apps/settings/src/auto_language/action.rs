//! 自动语言页的用户动作，不暴露进程标识编辑入口。

use qingjian_platform::InputLanguage;

use super::Target;

#[derive(Debug, Clone)]
pub enum Action {
    OpenPicker,

    ClosePicker,

    SelectApp(String),

    Choose(Target, Option<InputLanguage>),

    SavePicker,

    Remove(String),

    Preview,
}
