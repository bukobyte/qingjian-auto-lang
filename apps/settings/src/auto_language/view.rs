//! 自动语言页与最近十个应用选择面板，语言选择共用平台能力列表。

use std::sync::Arc;

use bamboo_slip::taffy::{
    AlignItems, Dimension, FlexDirection, JustifyContent, LengthPercentage, LengthPercentageAuto,
    NodeId, Position, Rect, Size, Style,
};
use bamboo_slip::{
    BoxPaint, Button, Fill, Interact, Menu, RenderError, ScrollArea, Select, SettingRow, Ui,
    Visual, dialog, group, paragraph, section_title,
};
use qingjian_platform::auto_language::{language_hint, language_label};
use qingjian_platform::{AppInfo, InputLanguage};

use super::{Action, Target};
use crate::i18n::{tr, tr_args};
use crate::message::{Message, Popup};
use crate::view::Page;
use crate::view::icons::Glyph;
use crate::view::parts::controls::toggle_info;

fn message(action: Action) -> Message {
    Message::AutoLanguage(action)
}

fn label(value: Option<InputLanguage>) -> &'static str {
    value.map_or(tr("auto_language.follow_default"), language_label)
}

fn select(
    ui: &mut Ui<Message>,
    page: &Page,
    target: Target,
    value: Option<InputLanguage>,
    placeholder: bool,
) -> Result<NodeId, RenderError> {
    Select::new(if placeholder {
        tr("auto_language.choose_language")
    } else {
        label(value)
    })
    .width(150.0)
    .build(
        ui,
        &page.theme,
        ("auto-language-select", target),
        Message::OpenMenu(Popup::AutoLanguage(target)),
    )
}

pub fn page(ui: &mut Ui<Message>, page: &Page) -> Result<Vec<NodeId>, RenderError> {
    let theme = &page.theme;
    let settings = page.settings;
    let state = &settings.auto_language;
    let config = &settings.config.auto_language;
    let mut out = vec![section_title(ui, theme, tr("page.auto_language"))?];
    let toggle = toggle_info(
        ui,
        page,
        tr("auto_language.enabled"),
        Some(tr("auto_language.enabled_info")),
        ("auto_language", "enabled"),
        config.enabled,
        !settings.config.general.english_mode,
    )?;
    let default = select(
        ui,
        page,
        Target::Default,
        Some(config.default_language),
        false,
    )?;
    let default = SettingRow::new(tr("auto_language.default"))
        .control(default)
        .build(ui, theme)?;
    out.push(group(ui, theme, &[toggle, default])?);
    out.push(section_title(ui, theme, tr("auto_language.current"))?);
    if let Some(app) = state.current() {
        let rule = config
            .rules
            .iter()
            .find(|rule| rule.app_id.eq_ignore_ascii_case(&app.app_id));
        let language = state
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.current_language);
        let detail = tr_args(
            "auto_language.actual",
            &[(
                "language",
                &language.map_or(tr("auto_language.unknown"), language_label),
            )],
        );
        let control = select(
            ui,
            page,
            Target::Current,
            rule.and_then(|rule| rule.language),
            rule.is_none(),
        )?;
        let row = app_row(ui, page, app, Some(&detail), &[control])?;
        out.push(group(ui, theme, &[row])?);
    } else {
        let text = if state.unavailable {
            tr("auto_language.unavailable")
        } else {
            tr("auto_language.empty")
        };
        out.push(paragraph(ui, theme, text, None)?);
    }
    out.push(section_title(ui, theme, tr("auto_language.rules"))?);
    let mut rows = Vec::new();
    for (index, rule) in config.rules.iter().enumerate() {
        let app = AppInfo {
            app_id: rule.app_id.clone(),
            display_name: rule.display_name.clone(),
            bundle_path: None,
        };
        let language = select(ui, page, Target::Rule(index), rule.language, false)?;
        let remove = Button::new(tr("auto_language.remove")).build(
            ui,
            theme,
            ("auto-language-remove", index),
            message(Action::Remove(rule.app_id.clone())),
        )?;
        let unavailable = rule
            .language
            .is_some_and(|language| !state.languages().contains(&language));
        rows.push(app_row(
            ui,
            page,
            &app,
            unavailable.then(|| tr("auto_language.language_unavailable")),
            &[language, remove],
        )?);
    }
    let add = Button::new(tr("auto_language.add")).build(
        ui,
        theme,
        "auto-language-add",
        message(Action::OpenPicker),
    )?;
    rows.push(
        SettingRow::new(tr("auto_language.recent"))
            .control(add)
            .build(ui, theme)?,
    );
    out.push(group(ui, theme, &rows)?);
    out.push(section_title(ui, theme, tr("auto_language.hints"))?);
    let hint = toggle_info(
        ui,
        page,
        tr("auto_language.show_hint"),
        None,
        ("auto_language", "show_switch_hint"),
        config.show_switch_hint,
        false,
    )?;
    let preview = Button::new(tr("auto_language.preview"))
        .disabled(!config.show_switch_hint)
        .build(ui, theme, "auto-language-preview", message(Action::Preview))?;
    let preview = SettingRow::new(tr("auto_language.hint_detail"))
        .control(preview)
        .build(ui, theme)?;
    out.push(group(ui, theme, &[hint, preview])?);
    out.push(paragraph(
        ui,
        theme,
        tr("auto_language.manual_detail"),
        None,
    )?);
    if let Some(("auto-language", notice)) = &settings.notice {
        out.push(paragraph(ui, theme, notice, None)?);
    }
    if state.picker_open {
        picker(ui, page)?;
    }
    Ok(out)
}

fn app_row(
    ui: &mut Ui<Message>,
    page: &Page,
    app: &AppInfo,
    detail: Option<&str>,
    controls: &[NodeId],
) -> Result<NodeId, RenderError> {
    let theme = &page.theme;
    let size = page.px(28.0);
    let visual = page
        .settings
        .auto_language
        .icons
        .get(&app.app_id)
        .map_or_else(
            || Visual::Icon {
                icon: Arc::new(Glyph::Language),
                size,
                color: theme.colors.secondary,
            },
            |icon| {
                Visual::Box(BoxPaint {
                    fill: Some(Fill::Image {
                        pixmap: Arc::clone(icon),
                        slice: None,
                        pixels_per_px: 1.0,
                    }),
                    border: None,
                    radius: page.px(5.0),
                })
            },
        );
    let icon = ui.scene().node(
        Style {
            size: Size {
                width: Dimension::length(size),
                height: Dimension::length(size),
            },
            flex_shrink: 0.0,
            ..Style::default()
        },
        visual,
        &[],
    )?;
    let title = paragraph(ui, theme, &app.display_name, None)?;
    let mut texts = vec![title];
    if let Some(detail) = detail {
        texts.push(
            ui.scene().node(
                Style::default(),
                Visual::Text {
                    text: detail.to_owned(),
                    style: theme
                        .text(theme.small, 400, theme.colors.secondary)
                        .wrapped(),
                },
                &[],
            )?,
        );
    }
    let text = ui.scene().node(
        Style {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            min_size: Size {
                width: LengthPercentageAuto::length(0.0),
                height: LengthPercentageAuto::auto(),
            },
            ..Style::default()
        },
        Visual::Group,
        &texts,
    )?;
    let mut children = vec![icon, text];
    children.extend(controls);
    ui.scene().node(
        Style {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::CENTER,
            gap: page.gap(10.0),
            padding: Rect {
                top: LengthPercentage::length(page.px(10.0)),
                bottom: LengthPercentage::length(page.px(10.0)),
                ..Rect::zero()
            },
            ..Style::default()
        },
        Visual::Group,
        &children,
    )
}

fn picker(ui: &mut Ui<Message>, page: &Page) -> Result<(), RenderError> {
    let theme = &page.theme;
    let state = &page.settings.auto_language;
    let mut children = vec![section_title(ui, theme, tr("auto_language.recent"))?];
    let recents = state
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.recent_apps.as_slice())
        .unwrap_or_default();
    let mut rows = Vec::new();
    for app in recents.iter().take(10) {
        let rule = page
            .settings
            .config
            .auto_language
            .rules
            .iter()
            .find(|rule| rule.app_id.eq_ignore_ascii_case(&app.app_id));
        let detail = rule.map(|rule| {
            tr_args(
                "auto_language.configured",
                &[("language", &label(rule.language))],
            )
        });
        let selected = state
            .selected
            .as_ref()
            .is_some_and(|selected| selected.app_id.eq_ignore_ascii_case(&app.app_id));
        let mark = if selected { "✓" } else { "" };
        let mark = paragraph(ui, theme, mark, None)?;
        let row = app_row(ui, page, app, detail.as_deref(), &[mark])?;
        ui.interact(
            row,
            ui.key(("auto-language-recent", &app.app_id)),
            Interact::new()
                .focusable()
                .on_click(message(Action::SelectApp(app.app_id.clone()))),
        );
        rows.push(row);
    }
    if rows.is_empty() {
        children.push(paragraph(
            ui,
            theme,
            if state.unavailable {
                tr("auto_language.unavailable")
            } else {
                tr("auto_language.empty")
            },
            None,
        )?);
    } else {
        let rows = group(ui, theme, &rows)?;
        children.push(
            ScrollArea::new(Style {
                size: Size {
                    width: Dimension::auto(),
                    height: Dimension::length(page.px(270.0)),
                },
                ..Style::default()
            })
            .build(ui, theme, "auto-language-recents-scroll", rows)?,
        );
    }
    children.push(paragraph(ui, theme, tr("auto_language.empty"), None)?);
    let language = select(ui, page, Target::Picker, state.chosen, false)?;
    children.push(
        SettingRow::new(tr("auto_language.enter_language"))
            .control(language)
            .build(ui, theme)?,
    );
    let close = Button::new(tr("auto_language.cancel")).build(
        ui,
        theme,
        "auto-language-picker-cancel",
        message(Action::ClosePicker),
    )?;
    let save = Button::new(tr("auto_language.save"))
        .primary()
        .disabled(state.selected.is_none())
        .build(
            ui,
            theme,
            "auto-language-picker-save",
            message(Action::SavePicker),
        )?;
    children.push(ui.scene().node(
        Style {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::END,
            gap: page.gap(8.0),
            ..Style::default()
        },
        Visual::Group,
        &[close, save],
    )?);
    dialog(ui, theme, "auto-language-picker", 440.0, &children)
}

pub fn menu(
    ui: &mut Ui<Message>,
    page: &Page,
    target: Target,
    window: (f32, f32),
) -> Result<Option<NodeId>, RenderError> {
    let Some(rect) = ui.rect(ui.key(("auto-language-select", target))) else {
        return Ok(None);
    };
    let state = &page.settings.auto_language;
    let mut values: Vec<Option<InputLanguage>> =
        state.languages().iter().copied().map(Some).collect();
    if target != Target::Default {
        values.push(None);
    }
    let labels: Vec<&str> = values.iter().map(|value| label(*value)).collect();
    let selected = values
        .iter()
        .position(|value| *value == state.value(target, &page.settings.config.auto_language));
    let menu = Menu::new(&labels, selected, rect, window).build(
        ui,
        &page.theme,
        "auto-language-menu",
        move |index| message(Action::Choose(target, values[index])),
        Message::CloseMenu,
    )?;
    // 选择面板也是 overlay，语言菜单必须在它之后绘制才能接到点击。
    ui.overlay(menu);
    Ok(None)
}

pub fn preview(ui: &mut Ui<Message>, page: &Page) -> Result<(), RenderError> {
    let Some((language, _)) = page.settings.auto_language.hint else {
        return Ok(());
    };
    let text = paragraph(ui, &page.theme, language_hint(language), None)?;
    let pad = LengthPercentage::length(page.px(16.0));
    let node = ui.scene().node(
        Style {
            position: Position::Absolute,
            inset: Rect {
                top: LengthPercentageAuto::length(page.px(45.0)),
                right: LengthPercentageAuto::length(page.px(18.0)),
                ..Rect::auto()
            },
            padding: Rect {
                left: pad,
                right: pad,
                top: pad,
                bottom: pad,
            },
            ..Style::default()
        },
        Visual::solid(page.theme.colors.popup, page.px(10.0)),
        &[text],
    )?;
    ui.overlay(node);
    Ok(())
}
