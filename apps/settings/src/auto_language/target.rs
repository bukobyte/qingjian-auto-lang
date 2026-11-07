//! 共用语言下拉框的修改目标。

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Target {
    Default,

    Current,

    Rule(usize),

    Picker,
}
