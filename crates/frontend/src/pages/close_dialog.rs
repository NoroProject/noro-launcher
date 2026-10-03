//! "The game is running — close anyway?"
//!
//! Closing the launcher stops the game it started (and any download), so the
//! window asks first. Minimizing keeps everything going.

use super::common::Cx;
use crate::components::btn;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight};
use i18n::t;

pub fn dialog(ui: &LauncherUI, cx: &mut Cx) -> Option<AnyElement> {
    if !ui.close_prompt {
        return None;
    }
    let game_running = ui.sync.values().any(|s| s.running);
    let body = if game_running {
        t("close-prompt-game")
    } else {
        t("close-prompt-download")
    };

    Some(
        div()
            .id("close-dialog")
            .absolute()
            .inset_0()
            .occlude()
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
                            .child(t("close-prompt-title")),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(TEXT_PRIMARY))
                            .child(body),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(8.))
                            .child(btn(
                                "close-minimize",
                                t("close-prompt-minimize"),
                                true,
                                cx.listener(|this, _e, window, cx| {
                                    this.close_prompt = false;
                                    window.minimize_window();
                                    cx.notify();
                                }),
                            ))
                            .child(btn(
                                "close-quit",
                                t("close-prompt-quit"),
                                false,
                                cx.listener(|_, _e, _w, cx| cx.quit()),
                            ))
                            .child(btn(
                                "close-cancel",
                                t("common-cancel"),
                                false,
                                cx.listener(|this, _e, _w, cx| {
                                    this.close_prompt = false;
                                    cx.notify();
                                }),
                            )),
                    ),
            )
            .into_any_element(),
    )
}
