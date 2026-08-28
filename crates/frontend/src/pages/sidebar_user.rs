use super::common::{parse_hex, Cx};
use crate::state::{LauncherUI, Page};
use crate::theme::*;
use gpui::{div, img, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight};
use i18n::t;

pub fn user_card(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    // With no profile loaded, say so outright: a stand-in like "Player" reads
    // as a real name, and a signed-out player doesn't notice they are.
    let username = ui
        .user
        .as_ref()
        .map(|u| u.username.clone())
        .unwrap_or_else(|| t("sidebar-signed-out"));
    let handle = ui
        .user
        .as_ref()
        .map(|u| match u.handle() {
            Some(name) => format!("@{name}"),
            None => u.username.clone(),
        })
        .unwrap_or_else(|| t("sidebar-no-identity"));

    div()
        .id("profile-card")
        .flex_1()
        .min_w_0()
        .flex()
        .items_center()
        .gap(px(8.))
        .px(px(6.))
        .py(px(4.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .hover(|d| d.bg(rgba(0xffffff0a)))
        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
            this.page = Page::Profile;
            // The animation loop stops while the profile is closed.
            this.start_skin_animation(cx);
            cx.notify();
        }))
        .child(avatar(ui))
        .child(identity(username, handle))
        .into_any_element()
}

pub fn user_avatar_only(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .id("profile-card-avatar")
        .size(px(40.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|d| d.bg(rgba(0xffffff15)))
        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
            this.page = Page::Profile;
            this.start_skin_animation(cx);
            cx.notify();
        }))
        .child(avatar(ui))
        .into_any_element()
}

fn avatar(ui: &LauncherUI) -> AnyElement {
    if let Some(avatar) = &ui.avatar_image {
        return img(avatar.clone())
            .size(px(32.))
            .rounded(px(R_SM))
            .flex_shrink_0()
            .into_any_element();
    }
    let username = ui
        .user
        .as_ref()
        .map(|u| u.username.clone())
        .unwrap_or_else(|| t("sidebar-signed-out"));
    let color = ui
        .user
        .as_ref()
        .and_then(|u| u.primary_color().map(parse_hex))
        .unwrap_or(ACCENT);
    let initial = username
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".into());
    div()
        .size(px(32.))
        .rounded(px(R_SM))
        .flex_shrink_0()
        .bg(rgb(color))
        .flex()
        .items_center()
        .justify_center()
        .text_color(rgb(0xffffff))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(14.))
        .font_weight(FontWeight::BOLD)
        .child(initial)
        .into_any_element()
}

fn identity(username: String, handle: String) -> AnyElement {
    div()
        // Without `flex_1` this shrinks to its content and truncates the name
        // even though there is room left before the icons.
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .child(
            div()
                .truncate()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(13.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(TEXT_PRIMARY))
                .child(username),
        )
        .child(
            div()
                .truncate()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .text_color(rgb(TEXT_MUTED))
                .child(handle),
        )
        .into_any_element()
}
