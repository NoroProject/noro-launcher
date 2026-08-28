//! Account card on the profile page: avatar, names and roles.

use super::common::{initial, panel, parse_hex, Cx};
use crate::components::{badge, btn};
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, img, prelude::*, px, rgb, AnyElement, FontWeight};
use i18n::t;

pub fn user_card(ui: &LauncherUI, user: &schema::UserProfile, cx: &mut Cx) -> AnyElement {
    let color = user.primary_color().map(parse_hex).unwrap_or(ACCENT);
    let roles: Vec<AnyElement> = user
        .roles
        .iter()
        .map(|r| {
            let c = r.color.as_deref().map(parse_hex).unwrap_or(ACCENT);
            badge(role_label(r), c).into_any_element()
        })
        .collect();

    panel()
        .p(px(20.))
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(avatar(ui, color))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(28.))
                .font_weight(FontWeight::EXTRA_BOLD)
                .text_color(rgb(TEXT_PRIMARY))
                .child(user.username.to_uppercase()),
        )
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(14.))
                .text_color(rgb(TEXT_MUTED))
                // A local account is created by an operator and has no platform
                // handle; show the in-game name rather than a bare "@".
                .child(match user.handle() {
                    Some(name) => format!("@{}", name.to_uppercase()),
                    None => user.username.to_uppercase(),
                }),
        )
        .child(div().flex().flex_wrap().gap(px(8.)).children(roles))
        .child(btn(
            "logout-profile",
            t("profile-sign-out"),
            false,
            cx.listener(|this, _e, _w, cx| {
                this.logout();
                cx.notify();
            }),
        ))
        .into_any_element()
}

fn avatar(ui: &LauncherUI, color: u32) -> AnyElement {
    if let Some(av) = &ui.avatar_image {
        img(av.clone())
            .size(px(80.))
            .rounded(px(R_SM))
            .into_any_element()
    } else {
        div()
            .size(px(80.))
            .rounded(px(R_SM))
            .bg(rgb(color))
            .flex()
            .items_center()
            .justify_center()
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(40.))
            .font_weight(FontWeight::BOLD)
            .text_color(rgb(0xffffff))
            .child(initial(
                &ui.user
                    .as_ref()
                    .map(|u| u.username.clone())
                    .unwrap_or_default(),
            ))
            .into_any_element()
    }
}

/// Pixel fonts have no Cyrillic, so fall back to the machine name for roles
/// whose display name isn't ASCII.
fn role_label(role: &schema::Role) -> String {
    if role.display_name.is_ascii() {
        role.display_name.to_uppercase()
    } else {
        role.name.to_uppercase()
    }
}
