use super::common::Cx;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight};
use i18n::t;
use schema::NotifLevel;

/// The title comes from the level: the master sends body text and nothing else.
fn style(level: NotifLevel) -> (&'static str, u32, &'static str) {
    match level {
        NotifLevel::Success => ("circle-check", SUCCESS, "toast-success"),
        NotifLevel::Warning => ("triangle-alert", WARNING, "toast-warning"),
        NotifLevel::Error => ("circle-alert", ERROR, "toast-error"),
        NotifLevel::Info => ("info", BLUE, "toast-info"),
    }
}

pub fn toast_overlay(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let Some(toast) = &ui.toast else {
        return div().into_any_element();
    };
    let (icon, color, title_key) = style(toast.level);

    div()
        .id("toast-container")
        .occlude()
        .on_click(|_, _, _| {})
        .absolute()
        .bottom(px(24.))
        .right(px(24.))
        .w(px(360.))
        .p(px(16.))
        .rounded(px(R_SM))
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(color))
        .flex()
        .items_start()
        .gap(px(12.))
        .child(div().flex_none().mt(px(2.)).child(ic(icon, 20., color)))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .text_size(px(12.))
                        .text_color(rgb(color))
                        .child(t(title_key)),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(TEXT_PRIMARY))
                        .child(toast.text.clone()),
                ),
        )
        .child(close_button(cx))
        .into_any_element()
}

/// There is no auto-dismiss timer, so this is the only way to clear a toast.
fn close_button(cx: &mut Cx) -> AnyElement {
    div()
        .id("toast-close")
        .flex_none()
        .size(px(24.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .hover(|d| d.bg(rgba(0xffffff14)))
        .child(ic("x", 14., TEXT_MUTED))
        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
            this.toast = None;
            cx.notify();
        }))
        .into_any_element()
}
