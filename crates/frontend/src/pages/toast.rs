// Over 150 lines: the stack and one card in it. A card alone has nowhere to say
// how it is dismissed.
//! Toasts: short-lived notices in the corner.
//!
//! A stack, not a single slot. Syncing can report three things in a row — a
//! live-reload, a locked file, a failed download — and with one slot only the
//! third was ever seen.
//!
//! They go away on their own; each carries its own timer (see
//! `LauncherUI::notify_toast`), and the length depends on how bad the news is.
//! Nothing here animates a countdown: a bar that shrinks means redrawing the
//! window for the whole life of the toast, and that is the cost of a decoration
//! nobody is watching.
//!
//! Anything worth coming back to belongs in the notification panel instead —
//! a toast is gone in seconds and cannot be recalled.

use super::common::Cx;
use crate::icons::ic;
use crate::state::{LauncherUI, Toast};
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use i18n::t;
use schema::NotifLevel;

/// Icon, accent colour and the word above the text.
fn style(level: NotifLevel) -> (&'static str, u32, &'static str) {
    match level {
        NotifLevel::Success => ("circle-check", SUCCESS, "toast-success"),
        NotifLevel::Warning => ("triangle-alert", WARNING, "toast-warning"),
        NotifLevel::Error => ("circle-alert", ERROR, "toast-error"),
        NotifLevel::Info => ("info", BLUE, "toast-info"),
    }
}

pub fn toast_overlay(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let cards: Vec<AnyElement> = ui.toasts.iter().map(|toast| card(toast, cx)).collect();

    div()
        .absolute()
        .bottom(px(20.))
        .right(px(20.))
        .flex()
        .flex_col()
        // Newest at the bottom, nearest the corner the eye is already in.
        .gap(px(8.))
        .children(cards)
        .into_any_element()
}

fn card(toast: &Toast, cx: &mut Cx) -> AnyElement {
    let (icon, colour, title_key) = style(toast.level);
    let id = toast.id;

    div()
        .id(SharedString::from(format!("toast-{id}")))
        .occlude()
        .w(px(380.))
        .rounded(px(R_MD))
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(BORDER))
        .flex()
        // Уровень читается по значку и заголовку. Цветная полоса слева, которая
        // была здесь до этого, вылезала за скруглённый угол карточки: GPUI
        // обрезает содержимое по радиусу не всегда, и держаться за украшение,
        // которое иногда торчит, незачем.
        .child(
            div()
                .flex_1()
                .min_w_0()
                .px(px(14.))
                .py(px(12.))
                .flex()
                .gap(px(10.))
                .child(div().pt(px(1.)).child(ic(icon, 15., colour)))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(3.))
                        .child(
                            div()
                                .font_family(FONT_PIXEL_ALT)
                                .text_size(px(11.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(colour))
                                .child(t(title_key)),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(rgb(TEXT_PRIMARY))
                                .child(toast.text.clone()),
                        ),
                )
                .child(close_button(id, cx)),
        )
        .into_any_element()
}

/// Dismiss by hand.
///
/// Its own button rather than "click anywhere on the card": a toast can carry
/// a path or an error worth selecting, and a card that vanishes under the
/// cursor takes that away.
fn close_button(id: u64, cx: &mut Cx) -> AnyElement {
    div()
        .id(SharedString::from(format!("toast-close-{id}")))
        .size(px(20.))
        .flex_shrink_0()
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|d| d.bg(rgba(0xffffff14)))
        .child(ic("x", 12., TEXT_MUTED))
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.dismiss_toast(id);
            cx.notify();
        }))
        .into_any_element()
}
