//! 单击 Shift 切换中 / 英（配置 `[shortcut] switch_mode` 勾了 shift 时）。
//!
//! 边沿从修饰键事件来：控制器在 `recognizedEvents:` 里声明了 flagsChanged
//! （缺省只回 `NSKeyDownMask`，不声明就一个修饰键事件都收不到，Shift 的按下 / 抬起只能靠轮询采样，
//! 而采样在「按下与抬起落在同一拍之间」时会把整次点击漏掉），拿到精确边沿后在这里判定
//! 「按下到抬起之间没有别的键插进来」= 一次单击。规则与 Windows 的 `com/key/tap.rs` 相同，但那边能看到
//! 被系统热键吃掉的键并作废这次单击，macOS 看不到（系统热键不进 `handleEvent:`），
//! 所以「按住 Shift 按了系统快捷键再松开」会被误判成一次单击。
//! 勾上之后中英由本模块决定，Caps Lock 退化成只管大小写。

use std::time::{Duration, Instant};

use crate::imk::modifiers;

use crate::host::Host;
use qingjian_platform::InputLanguage;

/// 只用单击 Shift 时沿用原来的防抖窗口；开了双击 Shift 后缩短，免得吞掉第二次按下。
const SINGLE_TAP_COOLDOWN: Duration = Duration::from_millis(150);

const DOUBLE_TAP_COOLDOWN: Duration = Duration::from_millis(50);

/// 按住超过这么久再松开不算单击：鼠标事件不进 IMK 会话，按住 Shift 拖选文字时没有别的键能作废这次单击。
const TAP_WINDOW: Duration = Duration::from_millis(300);

/// 两次完整的 Shift 单击落在这个窗口内，切换英文 / 第三输入法；超时才落实单击。
const DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(400);

/// 左右两个 Shift 的键码。只有 Shift 自己不算「插进来的别的键」：按下它本来就是这次单击的一部分；
/// 别的修饰键（Cmd / Ctrl / Option / Caps Lock）按下一律作废，与 Windows 的 `tap_key` 同口径。
fn is_shift_key(key_code: u16) -> bool {
    matches!(key_code, 56 | 60)
}

/// 单击 Shift 的判定与它切出来的持久英文模式。
#[derive(Default)]
pub struct ModeSwitch {
    /// 配置里勾了「单击 Shift」（`[shortcut] switch_mode` 含 shift）且没关掉英文模式。
    enabled: bool,

    /// 配置关掉了英文模式（`[general] english_mode = false`）：固定中文，Caps Lock 也不再切到英文。
    english_off: bool,

    /// Shift 按下之后还没有别的键插进来时，按下的时刻；抬起时没超过 [`TAP_WINDOW`] 就算一次单击。
    pending: Option<Instant>,

    /// 上次切换的时刻，[`TOGGLE_COOLDOWN`] 内的 Shift 边沿当抖动丢掉。
    toggled_at: Option<Instant>,

    /// 单击 Shift 切出来的持久英文模式；`enabled` 为假时无意义（`set_enabled(false)` 会清零）。
    english: bool,

    /// 配置了双击 Shift 的第三输入法。
    third_enabled: bool,

    /// 当前处在第三输入法（越南语）。
    vietnamese: bool,

    /// 中文单击 Shift 返回的上次外文；false 英文，true 越南语。
    last_foreign_vietnamese: bool,

    /// 第一下完整 Shift 单击等待第二下；超时后才落实成单击，避免双击时先闪一次中间状态。
    pending_tap: Option<Instant>,

    /// 自动规则接管后的持久状态；停用自动后也保留，避免与物理 Caps 状态交接时跳变。
    explicit_mode: bool,

    last_caps: bool,
}

impl ModeSwitch {
    /// 配置里勾了「单击 Shift」。
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// 单击 Shift 切出来的英文模式。
    pub fn english(&self) -> bool {
        self.english
    }

    pub fn vietnamese(&self) -> bool {
        self.vietnamese
    }

    pub fn english_off(&self) -> bool {
        self.english_off
    }

    pub fn set_english_off(&mut self, off: bool) {
        self.english_off = off;
        if off {
            self.english = false;
            self.vietnamese = false;
            self.reset();
        }
    }

    pub fn set_third_enabled(&mut self, enabled: bool) {
        self.third_enabled = enabled;
        if !enabled && !self.explicit_mode {
            self.vietnamese = false;
            self.pending_tap = None;
        }
    }

    /// 配置里的开关变了。
    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled == enabled {
            return;
        }
        self.enabled = enabled;
        // 关掉时退回中文，免得留着上次切出来的英文模式
        if !enabled && !self.explicit_mode {
            self.english = false;
            self.vietnamese = false;
            self.pending_tap = None;
        }
    }

    /// 焦点切换 / 停用：忘掉这次还没抬起的 Shift，
    /// 免得「按着 Shift 切走再切回、然后松开」被当成一次单击。
    pub fn reset(&mut self) {
        self.pending = None;
        self.pending_tap = None;
    }

    /// 规则只设置输入模式，不改变用户的 Shift 快捷键或物理 Caps Lock。
    pub fn set_language(&mut self, language: InputLanguage, caps: bool) {
        self.explicit_mode = true;
        self.last_caps = caps;
        self.english = language == InputLanguage::English;
        self.vietnamese = language == InputLanguage::Vietnamese;
        if self.english || self.vietnamese {
            self.last_foreign_vietnamese = self.vietnamese;
        }
        self.reset();
    }

    pub fn explicit_mode(&self) -> bool {
        self.explicit_mode
    }

    pub fn rebase_caps(&mut self, caps: bool) {
        self.last_caps = caps;
    }

    /// 没选 Shift 时 Caps 仍手动切中英，以边沿修改当前模式，不用灯位覆盖应用规则。
    pub fn sync_caps(&mut self, caps: bool) -> bool {
        let changed = caps != self.last_caps;
        self.last_caps = caps;
        if !changed || !self.explicit_mode || self.enabled || self.english_off {
            return false;
        }
        self.english = !self.english;
        self.vietnamese = false;
        self.last_foreign_vietnamese = false;
        self.reset();
        true
    }

    /// 任一普通按键按下：Shift 按下到抬起之间插了别的键，这次不算单击。
    pub fn note_key(&mut self) {
        self.pending = None;
    }

    pub fn has_pending_tap(&self) -> bool {
        self.pending_tap.is_some()
    }

    /// 400 ms 内没有第二次 Shift：勾了单击 Shift 才落实为“中文 ↔ 上次外文”。
    pub fn resolve_pending(&mut self) -> bool {
        self.resolve_pending_at(Instant::now())
    }

    fn resolve_pending_at(&mut self, now: Instant) -> bool {
        if self.pending.is_some() {
            return false;
        }
        if self
            .pending_tap
            .is_none_or(|at| now.duration_since(at) < DOUBLE_TAP_WINDOW)
        {
            return false;
        }
        self.pending_tap = None;
        if !self.enabled {
            return false;
        }
        self.single_switch();
        true
    }

    fn single_switch(&mut self) {
        if self.english {
            self.last_foreign_vietnamese = false;
            self.english = false;
        } else if self.vietnamese {
            self.last_foreign_vietnamese = true;
            self.vietnamese = false;
        } else if self.last_foreign_vietnamese && (self.third_enabled || self.explicit_mode) {
            self.vietnamese = true;
        } else {
            self.english = true;
        }
    }

    fn double_switch(&mut self) {
        if self.english {
            self.english = false;
            self.vietnamese = true;
            self.last_foreign_vietnamese = true;
        } else if self.vietnamese {
            self.vietnamese = false;
            self.english = true;
            self.last_foreign_vietnamese = false;
        } else if self.last_foreign_vietnamese {
            self.english = true;
            self.last_foreign_vietnamese = false;
        } else {
            self.vietnamese = true;
            self.last_foreign_vietnamese = true;
        }
    }

    /// 修饰键事件：`key_code` 那次按下（`down`）。构成一次「单击 Shift」返回 `true`。
    pub fn shift_event(&mut self, key_code: u16, down: bool) -> bool {
        self.shift_event_at(key_code, down, Instant::now())
    }

    fn shift_event_at(&mut self, key_code: u16, down: bool, now: Instant) -> bool {
        if self.english_off {
            self.reset();
            return false;
        }
        if !self.enabled && !self.third_enabled {
            return false;
        }
        // 别的修饰键（Cmd / Ctrl / Option / Caps Lock）：这次不算单击
        if !is_shift_key(key_code) {
            self.pending = None;
            return false;
        }
        let cooldown = if self.third_enabled {
            DOUBLE_TAP_COOLDOWN
        } else {
            SINGLE_TAP_COOLDOWN
        };
        if self
            .toggled_at
            .is_some_and(|at| now.duration_since(at) < cooldown)
        {
            self.pending = None;
            return false;
        }
        if down {
            self.pending = Some(now);
            return false;
        }
        let clicked = self
            .pending
            .take()
            .is_some_and(|at| now.duration_since(at) <= TAP_WINDOW);
        if clicked {
            if !self.third_enabled {
                if self.enabled {
                    self.single_switch();
                }
            } else if self
                .pending_tap
                .is_some_and(|at| now.duration_since(at) <= DOUBLE_TAP_WINDOW)
            {
                self.pending_tap = None;
                self.double_switch();
            } else {
                self.pending_tap = Some(now);
            }
            self.toggled_at = Some(now);
        }
        clicked && (!self.third_enabled || self.pending_tap.is_none())
    }
}

impl Host {
    /// 定时器处理 400 ms 内没有第二下的 Shift；没勾单击 Shift 时只清掉双击等待。
    pub fn resolve_pending_shift(&mut self) -> bool {
        let switched = self.mode_switch.resolve_pending();
        self.indicator
            .set_fast_poll(self.mode_switch.has_pending_tap());
        if !switched {
            return false;
        }
        self.note_manual_language();
        tracing::info!(
            english = self.english_mode(),
            vietnamese = self.vietnamese_mode(),
            "Shift 单击切换输入法"
        );
        self.refresh_mode_ui();
        self.render();
        true
    }

    /// 中英模式：关掉英文模式时固定中文；勾了「单击 Shift」时由 Shift 的切换决定（Caps Lock 退化成只管大小写），没勾看 Caps Lock。
    pub fn english_mode(&self) -> bool {
        if self.mode_switch.vietnamese() || self.mode_switch.english_off() {
            return false;
        }
        if self.mode_switch.enabled() || self.mode_switch.explicit_mode() {
            self.mode_switch.english()
        } else {
            modifiers::caps_lock_on()
        }
    }

    pub fn vietnamese_mode(&self) -> bool {
        !self.mode_switch.english_off() && self.mode_switch.vietnamese()
    }

    /// 英文模式下一个字母要不要大写：勾了「单击 Shift」时 Caps Lock 只管大小写（与 Shift 异或）；
    /// 没勾时 Caps Lock 就是中英开关，不参与大小写。
    pub fn english_letter_uppercase(&self) -> bool {
        if self.mode_switch.enabled() {
            modifiers::shift_down() != modifiers::caps_lock_on()
        } else {
            modifiers::shift_down()
        }
    }

    /// 收到了修饰键事件（`recognizedEvents:` 里的 flagsChanged）：Shift 的按下 / 抬起在这里判定，
    /// 这次切换了中英返回 `true`。
    pub fn shift_event(&mut self, key_code: u16, down: bool) -> bool {
        let switched = self.mode_switch.shift_event(key_code, down);
        self.indicator
            .set_fast_poll(self.mode_switch.has_pending_tap());
        if !switched {
            return false;
        }
        self.note_manual_language();
        tracing::info!(
            english = self.english_mode(),
            vietnamese = self.vietnamese_mode(),
            "Shift 切换输入法"
        );
        self.refresh_mode_ui();
        self.render();
        true
    }

    /// 中英模式或 Caps Lock 可能变了：刷新状态条与候选窗口的输入状态，变了就弹一次右上角提示。
    ///
    /// 中英变了显示「中」/「英」（优先）；只有大小写变了才显示「大写」/「小写」——后者出现在勾了
    /// 「单击 Shift」之后，那时 Caps Lock 退化成只管大小写。
    pub fn refresh_mode_ui(&mut self) {
        if !self.auto_language.needs_caps_rebase
            && self.mode_switch.sync_caps(modifiers::caps_lock_on())
        {
            self.note_manual_language();
        }
        let english = self.english_mode();
        let vietnamese = self.vietnamese_mode();
        self.indicator.update(english, vietnamese);
        let caps = modifiers::caps_lock_on();
        // 第一次只记不弹：启动或刚激活时不该凭空冒一个提示
        let language_changed = self
            .last_osd_vietnamese
            .is_some_and(|last| last != vietnamese)
            || self.last_osd_english.is_some_and(|last| last != english);
        if self.auto_language.enabled {
            if language_changed && self.settings.config().auto_language.show_switch_hint {
                self.osd
                    .show(qingjian_platform::auto_language::language_hint(
                        self.current_input_language(),
                    ));
            }
        } else if self
            .last_osd_vietnamese
            .is_some_and(|last| last != vietnamese)
        {
            self.osd.show(if vietnamese {
                "越"
            } else if english {
                "英"
            } else {
                "中"
            });
        } else if self.last_osd_english.is_some_and(|last| last != english) {
            self.osd.show(if english { "英" } else { "中" });
        } else if self.last_osd_caps.is_some_and(|last| last != caps) {
            self.osd.show(if caps { "大写" } else { "小写" });
        }
        self.last_osd_english = Some(english);
        self.last_osd_vietnamese = Some(vietnamese);
        self.last_osd_caps = Some(caps);
    }

    /// 停用时忘掉已提示过的状态：下次激活的第一次刷新只记不弹，焦点来回挪不至于弹提示。
    pub fn forget_osd_state(&mut self) {
        if !self.auto_language.enabled {
            self.last_osd_english = None;
            self.last_osd_vietnamese = None;
        }
        self.last_osd_caps = None;
    }

    /// 任一按键按下：除 Shift 自己以外的任何键（含别的修饰键）都作废待定的单击 Shift
    /// （Shift + 字母 / Shift + Cmd 组合都不该被当成单击）。
    pub fn note_key(&mut self, key_code: u16) -> bool {
        if !is_shift_key(key_code) {
            // 用户不等双击窗口就开始打字：勾了单击 Shift 才先切模式，再处理这个键。
            let switched = if self.mode_switch.has_pending_tap() && self.mode_switch.enabled() {
                self.mode_switch.pending_tap = None;
                self.mode_switch.single_switch();
                self.note_manual_language();
                self.refresh_mode_ui();
                true
            } else {
                self.mode_switch.pending_tap = None;
                false
            };
            self.indicator.set_fast_poll(false);
            self.mode_switch.note_key();
            return switched;
        }
        false
    }
}

#[cfg(test)]
mod tests;
