//! Small sidebar pieces: logo, bottom nav icons, empty state.

use super::common::Cx;
use crate::components::{mascot, pixel_title, Mood};
use crate::icons::ic;
use crate::state::{LauncherUI, Page};
use crate::theme::*;
use gpui::{div, img, prelude::*, px, rgb, rgba, AnyElement, App, ClickEvent, ObjectFit, Window};
use i18n::t;

pub fn nav_icon(
    id: &'static str,
    icon: &'static str,
    hint: String,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .tooltip(crate::components::hint(hint))
        .size(px(36.))
        .flex_shrink_0()
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .bg(if active {
            rgba((CTA << 8) | 0x18)
        } else {
            rgba(0x00000000)
        })
        .hover(|d| d.bg(rgba(0xffffff10)))
        .child(ic(icon, 16., if active { CTA } else { TEXT_MUTED }))
        .on_click(on_click)
        .into_any_element()
}

/// The operator's logo once it has loaded, the bundled one until then.
fn brand_logo(ui: &LauncherUI, size: f32) -> gpui::Img {
    match &ui.brand.logo {
        Some(image) => img(image.clone()),
        None => img("logo.png"),
    }
    .size(px(size))
    .object_fit(ObjectFit::Contain)
}

pub fn logo(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .id("home-logo")
        .cursor_pointer()
        .flex()
        .items_center()
        .gap(px(10.))
        .min_w_0()
        .hover(|d| d.opacity(0.8))
        .child(brand_logo(ui, 26.).flex_shrink_0())
        .child(
            div()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .child(pixel_title(ui.brand.name.to_uppercase(), 24., CTA)),
        )
        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
            this.page = this
                .selected_server_id()
                .map(Page::ServerDetail)
                .unwrap_or(Page::Servers);
            cx.notify();
        }))
        .into_any_element()
}

/// The logo in a collapsed sidebar, doubling as the expand button. On hover the
/// logo fades out and the icon takes its place at the same size — a corner badge
/// is unreadable at 40 px.
pub fn collapsed_logo_toggle(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .id("collapsed-logo-toggle")
        .tooltip(crate::components::hint(t("hint-expand-sidebar")))
        .group("collapsed-logo")
        .size(px(40.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|d| d.bg(rgba((CTA << 8) | 0x20)))
        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
            this.sidebar_collapsed = false;
            cx.notify();
        }))
        .child(
            div()
                .relative()
                .size(px(24.))
                .flex()
                .items_center()
                .justify_center()
                .child(brand_logo(ui, 24.).group_hover("collapsed-logo", |s| s.opacity(0.2)))
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .opacity(0.)
                        .group_hover("collapsed-logo", |s| s.opacity(1.))
                        .child(ic("panel-left-open", 20., CTA)),
                ),
        )
        .into_any_element()
}

/// Stands in for the server list. No border or fill: an outlined empty block
/// reads as a server card that failed to load.
pub fn empty_hint() -> AnyElement {
    div()
        .px(px(12.))
        .py(px(16.))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.))
        .child(mascot(Mood::Sleeping, 72.))
        .child(
            div()
                .text_center()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_MUTED))
                .child(t("sidebar-empty")),
        )
        .into_any_element()
}

/// A nav button with an unread dot.
///
/// A dot and not a number: the sidebar has room for one glyph, and "somebody
/// wrote" is the whole message — the count is in the list itself.
pub fn dot_icon(
    id: &'static str,
    icon: &'static str,
    hint: String,
    active: bool,
    unread: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .id(id)
        .tooltip(crate::components::hint(hint))
        .relative()
        .size(px(36.))
        .flex_shrink_0()
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .bg(if active {
            rgba((CTA << 8) | 0x18)
        } else {
            rgba(0x00000000)
        })
        .hover(|d| d.bg(rgba(0xffffff10)))
        .child(ic(icon, 16., if active { CTA } else { TEXT_MUTED }))
        .when(unread, |d| {
            d.child(
                div()
                    .absolute()
                    .top(px(6.))
                    .right(px(6.))
                    .size(px(7.))
                    .rounded(px(4.))
                    .bg(rgb(ACCENT)),
            )
        })
        .on_click(on_click)
        .into_any_element()
}
