//! 自动语言页的针对性截图与点击验收，使用临时配置和固定示例状态。

use std::path::Path;
use std::time::Instant;

use bamboo_slip::{App, Event, FontLibrary, Key, Runner};
use qingjian_platform::{AppInfo, Config, InputLanguage, LanguageRule, RuntimeSnapshot};

use crate::auto_language::{Action, Target};
use crate::message::{Message, Popup};
use crate::page_id::PageId;
use crate::settings::Settings;

const SCALE: f32 = 2.0;
const SIZE: (u32, u32) = (1800, 1360);

pub fn run(directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(directory)?;
    let path = directory.join("config.toml");
    Config::write_template_if_missing(&path)?;
    Config::set_bool(&path, "auto_language", "enabled", true)?;
    for (id, name, language) in [
        (
            "com.microsoft.VSCode",
            "Visual Studio Code",
            InputLanguage::English,
        ),
        ("com.openai.chat", "ChatGPT", InputLanguage::Chinese),
        ("com.apple.Terminal", "Terminal", InputLanguage::English),
    ] {
        Config::set_auto_language_rule(
            &path,
            &LanguageRule {
                app_id: id.to_owned(),
                display_name: name.to_owned(),
                language: Some(language),
            },
        )?;
    }
    let apps: Vec<AppInfo> = [
        ("com.microsoft.VSCode", "Visual Studio Code"),
        ("com.openai.chat", "ChatGPT"),
        ("com.apple.Terminal", "Terminal"),
        ("com.apple.finder", "访达"),
        ("com.apple.Notes", "备忘录"),
        ("com.apple.Safari", "Safari"),
        ("com.tinyspeck.slackmacgap", "Slack"),
        ("md.obsidian", "Obsidian"),
        ("com.apple.mail", "邮件"),
        ("com.apple.dt.Xcode", "Xcode"),
    ]
    .into_iter()
    .map(|(id, name)| AppInfo {
        app_id: id.to_owned(),
        display_name: name.to_owned(),
        bundle_path: None,
    })
    .collect();
    let mut settings = Settings::load(path.clone());
    settings.page = PageId::AutoLanguage;
    settings.auto_language.snapshot = Some(RuntimeSnapshot {
        session_id: "screenshot-only".to_owned(),
        current_app: apps.first().cloned(),
        recent_apps: apps,
        current_language: Some(InputLanguage::English),
        enabled: true,
        ..RuntimeSnapshot::default()
    });
    let mut runner = Runner::new(settings, FontLibrary::system("zh-CN")?);
    runner.stage().set_animations(false);
    shot(&mut runner, &directory.join("01-auto-language.png"))?;
    runner
        .app_mut()
        .update(Message::AutoLanguage(Action::OpenPicker));
    shot(&mut runner, &directory.join("02-recent-apps.png"))?;
    click(
        &mut runner,
        Key::ROOT.child(("auto-language-recent", "com.apple.finder")),
    )?;
    click(
        &mut runner,
        Key::ROOT.child(("auto-language-select", Target::Picker)),
    )?;
    shot(&mut runner, &directory.join("03-language-menu.png"))?;
    click(
        &mut runner,
        Key::ROOT.child("auto-language-menu").child(2_usize),
    )?;
    click(&mut runner, Key::ROOT.child("auto-language-picker-save"))?;
    let config = Config::load(&path)?;
    assert_eq!(
        config
            .auto_language
            .rules
            .iter()
            .find(|rule| rule.app_id == "com.apple.finder")
            .and_then(|rule| rule.language),
        Some(InputLanguage::Vietnamese)
    );
    assert_eq!(config.auto_language.rules.len(), 4);
    runner
        .app_mut()
        .update(Message::OpenMenu(Popup::AutoLanguage(Target::Current)));
    shot(&mut runner, &directory.join("04-current-language-menu.png"))?;
    runner.app_mut().update(Message::CloseMenu);
    runner
        .app_mut()
        .update(Message::AutoLanguage(Action::Preview));
    shot(&mut runner, &directory.join("05-language-hint.png"))?;
    println!(
        "Auto language page, recent-app selection, language menu, save and preview verified: {}",
        directory.display()
    );
    Ok(())
}

fn shot(runner: &mut Runner<Settings>, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let painted = runner.rebuild(SCALE, SIZE, Instant::now())?;
    painted.pixmap.save_png(path)?;
    Ok(())
}

fn click(runner: &mut Runner<Settings>, key: Key) -> Result<(), Box<dyn std::error::Error>> {
    runner.rebuild(SCALE, SIZE, Instant::now())?;
    let (x, y, width, height) = runner
        .interaction()
        .rect(key)
        .ok_or("Missing auto language control")?;
    runner.handle(&Event::PointerMoved {
        x: x + width / 2.0,
        y: y + height / 2.0,
    });
    runner.handle(&Event::PointerDown);
    runner.handle(&Event::PointerUp);
    runner.rebuild(SCALE, SIZE, Instant::now())?;
    Ok(())
}
