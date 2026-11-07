//! 模式切换与自动规则接管的针对性回归。

use std::time::{Duration, Instant};

use super::ModeSwitch;
use qingjian_platform::InputLanguage;

#[test]
fn automatic_mode_keeps_caps_manual_without_shift() {
    let mut switch = ModeSwitch::default();
    switch.set_language(InputLanguage::English, false);
    assert!(switch.english());
    assert!(!switch.sync_caps(false));
    assert!(switch.sync_caps(true));
    assert!(!switch.english());
    assert!(switch.sync_caps(false));
    assert!(switch.english());
}

#[test]
fn automatic_vietnamese_does_not_require_a_third_shortcut() {
    let mut switch = enabled();
    switch.set_language(InputLanguage::Vietnamese, true);
    switch.set_third_enabled(false);
    assert!(switch.vietnamese());
    assert!(!switch.sync_caps(false));
    switch.single_switch();
    assert!(!switch.vietnamese());
    switch.single_switch();
    assert!(switch.vietnamese());
}

#[test]
fn explicit_language_clears_pending_and_respects_disabled_input() {
    let mut switch = enabled();
    switch.pending_tap = Some(Instant::now());
    switch.set_language(InputLanguage::English, false);
    assert!(!switch.has_pending_tap());
    switch.set_enabled(false);
    assert!(switch.english());
    switch.set_english_off(true);
    assert!(!switch.english());
    assert!(!switch.vietnamese());
}

const LEFT_SHIFT: u16 = 56;

#[test]
fn disabling_input_switching_blocks_the_third_shortcut() {
    let mut switch = enabled();
    switch.set_third_enabled(true);
    switch.set_english_off(true);
    let start = Instant::now();
    for (ms, down) in [(0, true), (40, false), (100, true), (140, false)] {
        assert!(!switch.shift_event_at(LEFT_SHIFT, down, start + Duration::from_millis(ms)));
    }
    assert!(!switch.vietnamese());
    assert!(!switch.english());
}

#[test]
fn new_application_rebases_caps_without_a_manual_switch() {
    let mut switch = ModeSwitch::default();
    switch.set_language(InputLanguage::English, false);
    switch.rebase_caps(true);
    assert!(!switch.sync_caps(true));
    assert!(switch.english());
    assert!(switch.sync_caps(false));
    assert!(!switch.english());
}

fn enabled() -> ModeSwitch {
    let mut switch = ModeSwitch::default();
    switch.set_enabled(true);
    switch
}

#[test]
fn a_bounce_right_after_switching_is_ignored() {
    let mut switch = enabled();
    let start = Instant::now();
    let at = |ms| start + Duration::from_millis(ms);
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(0)));
    assert!(switch.shift_event_at(LEFT_SHIFT, false, at(80)));
    // 松开后 10 ms 补来的一对按下 / 抬起
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(90)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, false, at(95)));
    assert!(switch.english());
    // 冷却过后的下一次单击照常切回
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(400)));
    assert!(switch.shift_event_at(LEFT_SHIFT, false, at(480)));
    assert!(!switch.english());
}

#[test]
fn holding_shift_too_long_is_not_a_tap() {
    let mut switch = enabled();
    let start = Instant::now();
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, start));
    assert!(!switch.shift_event_at(LEFT_SHIFT, false, start + Duration::from_millis(900)));
    assert!(!switch.english());
}

#[test]
fn double_shift_toggles_the_third_input_method() {
    let mut switch = enabled();
    switch.set_third_enabled(true);
    let start = Instant::now();
    let at = |ms| start + Duration::from_millis(ms);
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(0)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, false, at(40)));
    assert!(!switch.english(), "第一下待定，不先闪英文");
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(90)));
    assert!(switch.shift_event_at(LEFT_SHIFT, false, at(120)));
    assert!(switch.vietnamese());
    assert!(!switch.english());
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(220)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, false, at(250)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(300)));
    assert!(switch.shift_event_at(LEFT_SHIFT, false, at(330)));
    assert!(switch.english());
    assert!(!switch.vietnamese());
}

#[test]
fn double_shift_works_without_single_shift_switching() {
    let mut switch = ModeSwitch::default();
    switch.set_third_enabled(true);
    let start = Instant::now();
    let at = |ms| start + Duration::from_millis(ms);
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(0)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, false, at(40)));
    assert!(!switch.english());
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(90)));
    assert!(switch.shift_event_at(LEFT_SHIFT, false, at(120)));
    assert!(switch.vietnamese());
}

#[test]
fn a_fast_human_double_tap_is_not_mistaken_for_keyboard_bounce() {
    let mut switch = enabled();
    switch.set_third_enabled(true);
    let start = Instant::now();
    let at = |ms| start + Duration::from_millis(ms);
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(0)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, false, at(30)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(80)));
    assert!(switch.shift_event_at(LEFT_SHIFT, false, at(110)));
    assert!(switch.vietnamese());
}

#[test]
fn cooldown_is_150ms_without_double_shift_and_50ms_with_it() {
    let start = Instant::now();
    let at = |ms| start + Duration::from_millis(ms);

    let mut single = enabled();
    assert!(!single.shift_event_at(LEFT_SHIFT, true, at(0)));
    assert!(single.shift_event_at(LEFT_SHIFT, false, at(60)));
    // 距上次切换 100 ms：没开双击时仍按原来的 150 ms 防抖丢掉。
    assert!(!single.shift_event_at(LEFT_SHIFT, true, at(160)));
    assert!(!single.shift_event_at(LEFT_SHIFT, false, at(210)));
    assert!(single.english());

    let mut double = enabled();
    double.set_third_enabled(true);
    assert!(!double.shift_event_at(LEFT_SHIFT, true, at(0)));
    assert!(!double.shift_event_at(LEFT_SHIFT, false, at(40)));
    // 距上次边沿超过 50 ms，且第二次抬起仍在首个抬起后的 400 ms 内。
    assert!(!double.shift_event_at(LEFT_SHIFT, true, at(100)));
    assert!(double.shift_event_at(LEFT_SHIFT, false, at(130)));
    assert!(double.vietnamese());
}

#[test]
fn single_shift_returns_to_the_last_foreign_language() {
    let mut switch = enabled();
    switch.set_third_enabled(true);
    let start = Instant::now();
    let at = |ms| start + Duration::from_millis(ms);

    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(0)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, false, at(30)));
    assert!(!switch.resolve_pending_at(at(429)));
    assert!(switch.resolve_pending_at(at(430)));
    assert!(switch.english());

    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(550)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, false, at(580)));
    assert!(switch.resolve_pending_at(at(980)));
    assert!(!switch.english());

    // 中文双击换到另一个外文（越南语），不经过英文中间态。
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(1100)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, false, at(1130)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(1380)));
    assert!(switch.shift_event_at(LEFT_SHIFT, false, at(1410)));
    assert!(switch.vietnamese());

    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(1520)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, false, at(1550)));
    assert!(switch.resolve_pending_at(at(1950)));
    assert!(!switch.vietnamese());
    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(2070)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, false, at(2100)));
    assert!(switch.resolve_pending_at(at(2500)));
    assert!(switch.vietnamese(), "中文单击回到上次外文越南语");
}

#[test]
fn shift_single_does_nothing_when_only_double_shift_is_enabled() {
    let mut switch = ModeSwitch::default();
    switch.set_third_enabled(true);
    let start = Instant::now();
    let at = |ms| start + Duration::from_millis(ms);

    assert!(!switch.shift_event_at(LEFT_SHIFT, true, at(0)));
    assert!(!switch.shift_event_at(LEFT_SHIFT, false, at(30)));
    assert!(!switch.resolve_pending_at(at(430)));
    assert!(!switch.english());
    assert!(!switch.vietnamese());
}
