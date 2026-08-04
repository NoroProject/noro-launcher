//! Profile page — user card + skin preview in two-column layout.

use super::common::{cabinet_url, page_title, panel, tabs, Cx};
use super::profile_asset::asset_panel;
use super::profile_skin::skin_card;
use super::profile_user::user_card;
use super::skin_drag;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement};
use i18n::t;

pub fn page(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .size_full()
        .relative()
        .bg(rgb(CONTENT_FALLBACK))
        .child(tabs(ui, cx))
        .child(
            div()
                .absolute()
                .top(px(104.))
                .left(px(40.))
                .right(px(40.))
                .bottom(px(40.))
                .flex()
                .flex_col()
                .gap(px(20.))
                .child(page_title(t("profile-title")))
                .child(match ui.user.clone() {
                    Some(_) => content(ui, cx),
                    None => empty_state(),
                }),
        )
        // Sits above the page so a skin drag survives leaving the preview box.
        .when(ui.skin_dragging, |d| d.child(skin_drag::drag_overlay(cx)))
        .into_any_element()
}

fn empty_state() -> AnyElement {
    panel()
        .p(px(20.))
        .text_color(rgb(TEXT_MUTED))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(14.))
        .child(t("profile-unavailable"))
        .into_any_element()
}

fn content(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .flex()
        .gap(px(24.))
        .child(left_column(ui, cx))
        .child(skin_card(ui, cx))
        .into_any_element()
}

fn left_column(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let url = cabinet_url(&ui.config.master_url);
    let user = ui.user.as_ref().expect("user in profile");

    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(user_card(ui, user, cx))
        .child(asset_panel(
            "edit-cape",
            t("profile-cape"),
            user.cape_url.as_deref(),
            &url,
            cx,
        ))
        .into_any_element()
}
