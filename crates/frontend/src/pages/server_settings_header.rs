//! The top of a server's settings page: where the values come from, reset and
//! the folder button.

use super::common::Cx;
use crate::icons::ic;
use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, InteractiveElement,
    SharedString,
};
use i18n::t;
use uuid::Uuid;

pub(super) fn page_header(
    server_id: Uuid,
    server_name: String,
    source: &'static str,
    reset_armed: bool,
    cx: &mut Cx,
) -> AnyElement {
    div()
        // As tall as the reset button, which only a local override shows:
        // sized by its content the row grew by 8px and pushed the panel down.
        .h(px(36.))
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
            d.child(reset_button(server_id, reset_armed, cx))
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

/// Resetting drops every per-server setting at once, so it takes two clicks.
fn reset_button(server_id: Uuid, armed: bool, cx: &mut Cx) -> AnyElement {
    div()
        .id(SharedString::from(format!("settings-reset-{server_id}")))
        .h(px(36.))
        .px(px(12.))
        .rounded(px(R_SM))
        .border_1()
        .border_color(rgb(if armed { ERROR } else { BORDER }))
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
                .child(if armed {
                    t("common-click-again")
                } else {
                    t("settings-reset")
                }),
        )
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            if this.confirm_or_arm(format!("reset-{server_id}"), cx) {
                this.reset_server_client_settings(server_id);
            }
            cx.notify();
        }))
        .into_any_element()
}

pub(super) fn open_folder_button(server_id: Uuid, cx: &mut Cx) -> AnyElement {
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
