//! "An admin is asking you to do something" modal: restart, cache reset, and so on.
//!
//! Four of the five actions have translation keys. `kill_game`, the per-action
//! descriptions and the "requested by" line have none yet — they are English
//! literals until the keys exist in noro-shared.

use super::common::Cx;
use crate::components::btn;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight};
use i18n::t;
use schema::RemoteAction;

pub fn dialog(ui: &LauncherUI, cx: &mut Cx) -> Option<AnyElement> {
    let prompt = ui.remote_action_prompt.as_ref()?;

    let (title, desc) = match prompt.action {
        RemoteAction::VerifyIntegrity => (
            t("remote-action-verify_integrity"),
            "The admin asked for an automatic file integrity check.",
        ),
        RemoteAction::ClearAssetCache => (
            t("remote-action-clear_asset_cache"),
            "The admin asks to clear the asset cache to sort out possible glitches.",
        ),
        RemoteAction::ReinstallBuild => (
            t("remote-action-reinstall_build"),
            "The admin asks to reinstall the current build from scratch.",
        ),
        RemoteAction::RestartLauncher => (
            t("remote-action-restart_launcher"),
            "The admin asked to restart the launcher.",
        ),
        RemoteAction::KillGame => (
            "Stop the game process".to_string(),
            "The admin asks to force-close the running Minecraft process.",
        ),
    };

    Some(
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba((OVERLAY << 8) | 0xcc))
            .child(
                div()
                    .w(px(460.))
                    .overflow_hidden()
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
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(TEXT_PRIMARY))
                            .child(format!("Requested by {}", prompt.actor_username)),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(TEXT_MUTED))
                            .child(desc),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(btn(
                                "remote-action-accept",
                                t("remote-action-accept"),
                                true,
                                cx.listener(|this, _e, _w, cx| {
                                    this.answer_remote_action(true);
                                    cx.notify();
                                }),
                            ))
                            .child(btn(
                                "remote-action-decline",
                                t("remote-action-decline"),
                                false,
                                cx.listener(|this, _e, _w, cx| {
                                    this.answer_remote_action(false);
                                    cx.notify();
                                }),
                            )),
                    ),
            )
            .into_any_element(),
    )
}
