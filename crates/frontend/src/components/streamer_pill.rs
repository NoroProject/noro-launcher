//! "Streamer mode is on", in the window bar on every page.
//!
//! A mode that hides things has to say it is on: otherwise a missing handle or
//! a masked console line reads as a bug. Clicking the pill turns it off.

use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, Context, FontWeight};

pub fn streamer_pill(ui: &LauncherUI, cx: &mut Context<LauncherUI>) -> Option<AnyElement> {
    if !ui.streamer {
        return None;
    }
    Some(
        div()
            .id("streamer-pill")
            .h(px(24.))
            .flex()
            .items_center()
            .gap(px(8.))
            .px(px(8.))
            .rounded(px(R_SM))
            .cursor_pointer()
            .bg(rgba((ACCENT << 8) | 0x22))
            .border_1()
            .border_color(rgb(ACCENT))
            .hover(|s| s.bg(rgba((ACCENT << 8) | 0x44)))
            .child(ic("video", 12., ACCENT))
            .child(
                div()
                    .text_size(px(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(ACCENT))
                    .child(i18n::t("streamer-pill")),
            )
            .child(ic("x", 12., ACCENT))
            .on_click(cx.listener(|this, _e, _w, cx| {
                this.set_streamer(false, cx);
                cx.notify();
            }))
            .into_any_element(),
    )
}
