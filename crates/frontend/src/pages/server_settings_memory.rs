//! The memory row of a server's settings: minimum and maximum with their steppers.

use super::common::Cx;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, InteractiveElement,
    SharedString,
};
use i18n::t;
use uuid::Uuid;

pub(super) fn memory(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let settings = ui.server_client_settings(server_id);
    let warning = ui.memory_warning(settings.memory_max_mb);
    div()
        .max_w(px(260.))
        .flex()
        .flex_col()
        .items_center()
        .items_end()
        .gap(px(8.))
        .child(mem_group(
            "min",
            t("settings-memory-min"),
            settings.memory_min_mb,
            true,
            server_id,
            cx,
        ))
        .child(mem_group(
            "max",
            t("settings-memory-max"),
            settings.memory_max_mb,
            false,
            server_id,
            cx,
        ))
        .children(warning.map(super::settings_rows::warning_line))
        .into_any_element()
}

/// `id` stays untranslated, or the button ids would shift with the language.
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

fn adjust(ui: &mut LauncherUI, server_id: Uuid, is_min: bool, delta: i32) {
    let ceiling = ui.memory_ceiling_mb() as i32;
    let settings = ui.server_client_settings(server_id);
    let mut min = settings.memory_min_mb as i32;
    let mut max = settings.memory_max_mb as i32;
    if is_min {
        min = (min + delta).clamp(512, ceiling);
    } else {
        max = (max + delta).clamp(512, ceiling);
    }
    if max < min {
        max = min;
    }
    ui.set_server_memory(server_id, min as u32, max as u32);
}
