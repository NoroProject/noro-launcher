//! Streamer mode in the settings: the switch, and whether a streaming app
//! flips it.

use super::common::{panel, Cx};
use super::settings_rows::row;
use crate::components::checkbox_row;
use crate::state::LauncherUI;
use gpui::{prelude::*, AnyElement, ClickEvent, SharedString};
use i18n::{t, t_args, FluentArgs};

pub fn streamer_panel(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let mut args = FluentArgs::new();
    args.set("keys", crate::STREAMER_KEYS);
    panel()
        .flex()
        .flex_col()
        .child(row(
            "video",
            t("settings-streamer"),
            t_args("settings-streamer-hint", &args),
            checkbox_row(
                SharedString::new_static("g-streamer"),
                ui.streamer,
                true,
                cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    let on = !this.streamer;
                    this.set_streamer(on, cx);
                    cx.notify();
                }),
            )
            .into_any_element(),
            true,
        ))
        .child(row(
            "radio",
            t("settings-streamer-auto"),
            t("settings-streamer-auto-hint"),
            checkbox_row(
                SharedString::new_static("g-streamer-auto"),
                ui.config.streamer_auto,
                true,
                cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    let v = this.config.streamer_auto;
                    this.set_streamer_auto(!v);
                    cx.notify();
                }),
            )
            .into_any_element(),
            false,
        ))
        .into_any_element()
}
