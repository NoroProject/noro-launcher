//! Mod page: markdown description, screenshots, metadata.
//!
//! Search results carry only a title and one line of description, so the full
//! page is a separate request that lands in `ui.mod_project`. Until it arrives
//! the short description stands in for it.

use super::common::Cx;
use super::mod_detail_body::description;
use super::mod_detail_parts::{gallery, tab_button};
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::CatalogHitInfo;
use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent, FontWeight};
use i18n::t;
use uuid::Uuid;

pub fn view(ui: &mut LauncherUI, server_id: Uuid, hit: CatalogHitInfo, cx: &mut Cx) -> AnyElement {
    let project = ui.mod_project.clone();
    let shots = project
        .as_ref()
        .map(|p| p.gallery.clone())
        .unwrap_or_default();
    for url in &shots {
        ui.ensure_screenshot_loaded(Some(url.clone()), cx);
    }

    let show_gallery = ui.mod_detail_gallery && !shots.is_empty();

    div()
        .flex_1()
        .min_h_0()
        .rounded(px(R_MD))
        .bg(rgb(BG_PANEL))
        .border_1()
        .border_color(rgb(BORDER))
        .p(px(32.))
        .flex()
        .flex_col()
        .gap(px(20.))
        .child(header(ui, server_id, &hit, cx))
        .child(tab_bar(shots.len(), show_gallery, cx))
        .child(if show_gallery {
            gallery(ui, &shots)
        } else {
            description(&hit, project.as_ref())
        })
        .into_any_element()
}

/// Title, one line of facts, and the action.
///
/// The action is the point of the page and was missing from it: a mod could be
/// read about here but not installed, so the only way to add one was to go back
/// to the list and find it again. It is the same control as on a card, at the
/// size a primary action deserves.
fn header(ui: &LauncherUI, server_id: Uuid, hit: &CatalogHitInfo, cx: &mut Cx) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(16.))
        .child(super::content_card::mod_avatar(ui, &hit.icon_url))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(if hit.title.len() > 34 {
                            px(16.)
                        } else {
                            px(20.)
                        })
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(CTA))
                        .truncate()
                        .child(hit.title.clone()),
                )
                .child(facts(hit)),
        )
        .child(super::content_card::detail_action(ui, server_id, hit, cx))
        .into_any_element()
}

/// Author and downloads, the way they are written on a card.
///
/// `Provider: MODRINTH | Downloads: 231674837` was three mistakes in one line:
/// the provider is chosen by the tab above, English labels sat in the middle of
/// a Russian interface, and nobody reads nine digits — they read «231.7M».
fn facts(hit: &CatalogHitInfo) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .text_size(px(11.))
        .text_color(rgb(TEXT_MUTED))
        .children(
            hit.author
                .clone()
                .map(|author| div().child(format!("{} {author}", t("mods-by")))),
        )
        .when(hit.downloads > 0, |d| {
            d.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .child(ic("download", 11., TEXT_MUTED))
                    .child(super::content_card::compact_downloads(hit.downloads)),
            )
        })
        .into_any_element()
}

fn tab_bar(shots: usize, gallery_active: bool, cx: &mut Cx) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .border_b_1()
        .border_color(rgb(BORDER))
        .pb(px(12.))
        .child(tab_button(
            "mod-tab-description",
            &t("content-tab-description"),
            !gallery_active,
            cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.mod_detail_gallery = false;
                cx.notify();
            }),
        ))
        .when(shots > 0, |d| {
            d.child(tab_button(
                "mod-tab-gallery",
                &format!("{} ({shots})", t("content-tab-gallery")),
                gallery_active,
                cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.mod_detail_gallery = true;
                    cx.notify();
                }),
            ))
        })
        .into_any_element()
}
