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
            t("remote-action-desc-verify_integrity"),
        ),
        RemoteAction::ClearAssetCache => (
            t("remote-action-clear_asset_cache"),
            t("remote-action-desc-clear_asset_cache"),
        ),
        RemoteAction::ReinstallBuild => (
            t("remote-action-reinstall_build"),
            t("remote-action-desc-reinstall_build"),
        ),
        RemoteAction::RestartLauncher => (
            t("remote-action-restart_launcher"),
            t("remote-action-desc-restart_launcher"),
        ),
        RemoteAction::KillGame => (
            t("remote-action-kill_game"),
            t("remote-action-desc-kill_game"),
        ),
    };

    Some(
        div()
            // Takes the clicks: without it they went through to the Play
            // button and the sidebar under the dialog.
            .id("remote-action-dialog")
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
                            .child({
                                let mut args = i18n::FluentArgs::new();
                                args.set("name", prompt.actor_username.clone());
                                i18n::t_args("remote-action-requested-by", &args)
                            }),
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
