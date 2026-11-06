//! What start-up found wrong with the game files folder: a disk that is not
//! connected, or a move that was interrupted. Asked once, over everything,
//! because launching before the answer would download into the wrong place.

use super::common::Cx;
use crate::components::btn;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::DataStatus;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight, SharedString};
use i18n::{t, t_args, FluentArgs};

pub fn dialog(ui: &LauncherUI, cx: &mut Cx) -> Option<AnyElement> {
    if ui.data.prompt_dismissed || ui.data.moving() {
        return None;
    }
    let view = ui.data.view.as_ref()?;
    let mut args = FluentArgs::new();
    let (title, body, hint, buttons) = match &view.status {
        DataStatus::Ready => return None,
        DataStatus::Unavailable => {
            args.set("path", view.path.display().to_string());
            (
                t("data-unavailable-title"),
                t_args("data-unavailable-body", &args),
                t_args("data-unavailable-reset-hint", &args),
                unavailable_buttons(cx),
            )
        }
        DataStatus::Interrupted { from, to } => {
            args.set("from", from.display().to_string());
            args.set("to", to.display().to_string());
            (
                t("data-interrupted-title"),
                t_args("data-interrupted-body", &args),
                t_args("data-interrupted-rollback-hint", &args),
                interrupted_buttons(cx),
            )
        }
    };
    Some(
        div()
            .id("data-dialog")
            .occlude()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba((OVERLAY << 8) | 0xcc))
            .child(
                div()
                    .w(px(460.))
                    .bg(rgb(BG_PANEL))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .rounded(px(12.))
                    .p(px(24.))
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(
                        div()
                            .font_family(FONT_PIXEL_ALT)
                            .text_size(px(15.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(CTA))
                            .child(title),
                    )
                    .child(text(body, TEXT_PRIMARY))
                    .child(text(hint, TEXT_MUTED))
                    .child(buttons),
            )
            .into_any_element(),
    )
}

fn text(body: impl Into<SharedString>, color: u32) -> AnyElement {
    div()
        .text_size(px(12.))
        .text_color(rgb(color))
        .child(body.into())
        .into_any_element()
}

fn unavailable_buttons(cx: &mut Cx) -> AnyElement {
    div()
        .flex()
        .gap(px(8.))
        .child(btn(
            "data-retry",
            t("data-unavailable-retry"),
            true,
            cx.listener(|this, _e, _w, cx| {
                this.retry_data_location();
                cx.notify();
            }),
        ))
        .child(btn(
            "data-reset",
            t("data-unavailable-reset"),
            false,
            cx.listener(|this, _e, _w, cx| {
                this.reset_data_location();
                cx.notify();
            }),
        ))
        .child(btn(
            "data-later",
            t("data-unavailable-later"),
            false,
            cx.listener(|this, _e, _w, cx| {
                this.data.prompt_dismissed = true;
                cx.notify();
            }),
        ))
        .into_any_element()
}

fn interrupted_buttons(cx: &mut Cx) -> AnyElement {
    div()
        .flex()
        .gap(px(8.))
        .child(btn(
            "data-dialog-resume",
            t("data-interrupted-resume"),
            true,
            cx.listener(|this, _e, _w, cx| {
                this.resolve_data_move(true);
                cx.notify();
            }),
        ))
        .child(btn(
            "data-dialog-rollback",
            t("data-interrupted-rollback"),
            false,
            cx.listener(|this, _e, _w, cx| {
                this.resolve_data_move(false);
                cx.notify();
            }),
        ))
        .into_any_element()
}
