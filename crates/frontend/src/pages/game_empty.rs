//! The "no servers" screen.

use super::common::{tabs, Cx};
use crate::components::{mascot, Mood};
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, FontWeight};
use i18n::t;

pub fn empty(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .size_full()
        .relative()
        .bg(rgb(CONTENT_FALLBACK))
        .child(tabs(ui, cx))
        .child(
            div()
                .absolute()
                .top(px(112.))
                .left(px(40.))
                .right(px(40.))
                .bottom(px(40.))
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(16.))
                .child(mascot(Mood::Thinking, 132.))
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(20.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(TEXT_PRIMARY))
                        .child(t("game-no-servers")),
                )
                .child(
                    div()
                        .max_w(px(360.))
                        .text_center()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(13.))
                        .text_color(rgb(TEXT_MUTED))
                        .child(t("game-no-servers-hint")),
                ),
        )
        .into_any_element()
}
