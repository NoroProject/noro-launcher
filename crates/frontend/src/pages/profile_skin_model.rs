//! Skin model: classic (Steve) or slim (Alex).
//!
//! Switching doesn't touch the image, only the arm width. It can't require a
//! re-upload either: a skin that arrived by nickname was never on the player's
//! disk to begin with.

use super::common::Cx;
use crate::components::btn;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement};
use i18n::t;

/// `None` when the player has no skin — there is nothing to switch.
pub fn model_row(ui: &LauncherUI, cx: &mut Cx) -> Option<AnyElement> {
    ui.user.as_ref()?.skin_url.as_ref()?;
    let slim = ui.user.as_ref().is_some_and(|u| u.skin_slim);

    Some(
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                div()
                    .font_family(FONT_PIXEL_ALT)
                    .text_size(px(12.))
                    .text_color(rgb(TEXT_MUTED))
                    .child(t("profile-skin-model")),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(choice(ui, cx, false, !slim, "profile-skin-model-classic"))
                    .child(choice(ui, cx, true, slim, "profile-skin-model-slim")),
            )
            .into_any_element(),
    )
}

fn choice(
    ui: &LauncherUI,
    cx: &mut Cx,
    slim: bool,
    active: bool,
    label_key: &'static str,
) -> AnyElement {
    // While an upload is in flight the click is dropped rather than the
    // button disabled.
    let busy = ui.skin_uploading;
    div()
        .flex_1()
        .child(btn(
            if slim {
                "skin-model-slim"
            } else {
                "skin-model-classic"
            },
            t(label_key),
            active,
            cx.listener(move |this, _, _, cx| {
                if !busy {
                    this.set_skin_model(slim, cx);
                }
            }),
        ))
        .into_any_element()
}
