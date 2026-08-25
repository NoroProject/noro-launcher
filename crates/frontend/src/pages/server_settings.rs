//! Настройки клиента для конкретного сервера (JVM, консоль, флаги).
use super::common::{panel, tabs, Cx};
use crate::components::checkbox_row;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, InteractiveElement,
    SharedString,
};
use i18n::t;
use uuid::Uuid;

pub fn page(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let source = settings_source(ui, server_id);
    let server_name = ui
        .server(&server_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "Server".into());
    div()
        .size_full()
        .relative()
        .bg(rgb(CONTENT_FALLBACK))
        .child(tabs(ui, cx))
        .child(
            div()
                .absolute()
                .top(px(92.))
                .left(px(32.))
                .right(px(32.))
                .bottom(px(32.))
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(page_header(server_id, server_name, source, cx))
                .child(settings_panel(ui, server_id, cx)),
        )
        .into_any_element()
}

/// Ключ перевода источника настроек. Именно ключ, а не текст: по нему
/// сравнивают состояние, и перевод не должен на это влиять.
fn settings_source(ui: &LauncherUI, server_id: Uuid) -> &'static str {
    if ui.has_server_client_override(server_id) {
        "settings-source-override"
    } else if ui.server_recommendations.contains_key(&server_id) {
        "settings-source-recommended"
    } else {
        "settings-source-default"
    }
}

fn page_header(
    server_id: Uuid,
    server_name: String,
    source: &'static str,
    cx: &mut Cx,
) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .mb(px(8.))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(18.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(t("settings-client-title")),
        )
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_MUTED))
                .child(server_name),
        )
        .child(source_pill(source))
        .child(div().flex_1())
        .when(source == "settings-source-override", |d| {
            d.child(reset_button(server_id, cx))
        })
        .into_any_element()
}

fn source_pill(source: &'static str) -> AnyElement {
    let color = if source == "settings-source-override" {
        ACCENT
    } else {
        CTA
    };
    div()
        .h(px(28.))
        .px(px(10.))
        .rounded(px(R_SM))
        .bg(rgba((color << 8) | 0x18))
        .border_1()
        .border_color(rgba((color << 8) | 0x44))
        .flex()
        .items_center()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(10.))
        .text_color(rgb(color))
        .child(t(source))
        .into_any_element()
}

fn reset_button(server_id: Uuid, cx: &mut Cx) -> AnyElement {
    div()
        .id(SharedString::from(format!("settings-reset-{server_id}")))
        .h(px(36.))
        .px(px(12.))
        .rounded(px(R_SM))
        .border_1()
        .border_color(rgb(BORDER))
        .bg(rgb(BG_CARD))
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .cursor_pointer()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(ic("rotate-ccw", 14., TEXT_SECONDARY))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(t("settings-reset")),
        )
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.reset_server_client_settings(server_id);
            cx.notify();
        }))
        .into_any_element()
}
fn open_folder_button(server_id: Uuid, cx: &mut Cx) -> AnyElement {
    div()
        .id(SharedString::from(format!("settings-folder-{server_id}")))
        .h(px(36.))
        .px(px(12.))
        .rounded(px(R_SM))
        .border_1()
        .border_color(rgb(BORDER))
        .bg(rgb(BG_CARD))
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .cursor_pointer()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(t("settings-folder-open")),
        )
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.open_server_client_folder(server_id);
            cx.notify();
        }))
        .into_any_element()
}

fn settings_panel(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    panel()
        .overflow_hidden()
        .flex()
        .flex_col()
        .child(setting_row(
            "memory-stick",
            t("settings-memory"),
            t("settings-memory-hint"),
            memory(ui, server_id, cx),
            true,
        ))
        .child(setting_row(
            "terminal",
            t("settings-console"),
            t("settings-console-hint"),
            console(ui, server_id, cx),
            true,
        ))
        .child(setting_row(
            "maximize",
            t("settings-fullscreen"),
            t("settings-fullscreen-hint"),
            fullscreen(ui, server_id, cx),
            true,
        ))
        .child(setting_row(
            "code",
            t("settings-jvm-flags"),
            t("settings-jvm-hint"),
            flags(ui, server_id),
            true,
        ))
        .child(setting_row(
            "folder",
            t("settings-folder"),
            t("settings-folder-hint"),
            folder(ui, server_id, cx),
            false,
        ))
        .into_any_element()
}

fn setting_row(
    icon: &'static str,
    title: impl Into<gpui::SharedString>,
    subtitle: impl Into<gpui::SharedString>,
    control: AnyElement,
    border: bool,
) -> AnyElement {
    div()
        .min_h(px(72.))
        .px(px(20.))
        .py(px(14.))
        .when(border, |d| d.border_b_1().border_color(rgb(BORDER)))
        .flex()
        .items_center()
        .gap(px(16.))
        .child(
            div()
                .w(px(32.))
                .h(px(32.))
                .flex_none()
                .rounded(px(R_SM))
                .bg(rgba(0xffffff0c))
                .border_1()
                .border_color(rgb(BORDER))
                .flex()
                .items_center()
                .justify_center()
                .child(ic(icon, 16., TEXT_SECONDARY)),
        )
        .child(row_label(title, subtitle))
        .child(div().flex_1())
        .child(control)
        .into_any_element()
}

fn row_label(
    title: impl Into<gpui::SharedString>,
    subtitle: impl Into<gpui::SharedString>,
) -> AnyElement {
    let (title, subtitle) = (title.into(), subtitle.into());
    div()
        .w(px(300.))
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(TEXT_SECONDARY))
                .child(title),
        )
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_MUTED))
                .child(subtitle),
        )
        .into_any_element()
}

fn memory(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let settings = ui.server_client_settings(server_id);
    div()
        .max_w(px(260.))
        .flex()
        .flex_col()
        .items_center()
        .items_end()
        .gap(px(8.))
        .child(mem_group(
            "min",
            "MIN",
            settings.memory_min_mb,
            true,
            server_id,
            cx,
        ))
        .child(mem_group(
            "max",
            "MAX",
            settings.memory_max_mb,
            false,
            server_id,
            cx,
        ))
        .into_any_element()
}

/// `id` не переводится, иначе идентификаторы кнопок менялись бы вместе с языком.
fn mem_group(
    id: &'static str,
    label: impl Into<gpui::SharedString>,
    value: u32,
    is_min: bool,
    server_id: Uuid,
    cx: &mut Cx,
) -> AnyElement {
    div()
        .h(px(36.))
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .w(px(32.))
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .text_color(rgb(TEXT_MUTED))
                .child(label.into()),
        )
        .child(mini_icon_btn(
            SharedString::from(format!("cs-mem-dec-{id}")),
            "minus",
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                adjust(this, server_id, is_min, -512);
                cx.notify();
            }),
        ))
        .child(
            div()
                .w(px(76.))
                .h(px(36.))
                .rounded(px(R_SM))
                .bg(rgba(0x00000024))
                .border_1()
                .border_color(rgb(BORDER))
                .flex()
                .items_center()
                .justify_center()
                .text_center()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .font_weight(FontWeight::BOLD)
                .child(format!("{value} MB")),
        )
        .child(mini_icon_btn(
            SharedString::from(format!("cs-mem-inc-{id}")),
            "plus",
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                adjust(this, server_id, is_min, 512);
                cx.notify();
            }),
        ))
        .into_any_element()
}

fn mini_icon_btn(
    id: SharedString,
    icon: &'static str,
    on_click: impl Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .size(px(36.))
        .rounded(px(R_SM))
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(BORDER))
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .child(ic(icon, 14., TEXT_SECONDARY))
        .on_click(on_click)
        .into_any_element()
}

fn console(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let enabled = ui.server_client_settings(server_id).show_console_on_launch;
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(checkbox_row(
            SharedString::from(format!("cs-console-on-launch-{server_id}")),
            enabled,
            true,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                let v = this
                    .server_client_settings(server_id)
                    .show_console_on_launch;
                this.set_server_show_console_on_launch(server_id, !v);
                cx.notify();
            }),
        ))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(t("settings-console-open")),
        )
        .into_any_element()
}

fn fullscreen(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let enabled = ui.server_client_settings(server_id).fullscreen;
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(checkbox_row(
            SharedString::from(format!("cs-fullscreen-{server_id}")),
            enabled,
            true,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                let v = this.server_client_settings(server_id).fullscreen;
                this.set_server_fullscreen(server_id, !v);
                cx.notify();
            }),
        ))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(t("settings-fullscreen-show")),
        )
        .into_any_element()
}
fn folder(_ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(open_folder_button(server_id, cx))
        .into_any_element()
}

fn flags(ui: &LauncherUI, server_id: Uuid) -> AnyElement {
    let settings = ui.server_client_settings(server_id);
    let (color, text) = if settings.jvm_flags.is_empty() {
        (TEXT_MUTED, "not set".to_string())
    } else {
        (TEXT_SECONDARY, settings.jvm_flags)
    };
    div()
        .font_family("Courier New")
        .text_size(px(12.))
        .text_color(rgb(color))
        .child(text)
        .into_any_element()
}

fn adjust(ui: &mut LauncherUI, server_id: Uuid, is_min: bool, delta: i32) {
    let settings = ui.server_client_settings(server_id);
    let mut min = settings.memory_min_mb as i32;
    let mut max = settings.memory_max_mb as i32;
    if is_min {
        min = (min + delta).clamp(512, 65536);
    } else {
        max = (max + delta).clamp(512, 65536);
    }
    if max < min {
        max = min;
    }
    ui.set_server_memory(server_id, min as u32, max as u32);
}
