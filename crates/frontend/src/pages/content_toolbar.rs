// Over 150 lines: every control of the browser's top bar, and they only make
// sense as one row — each one re-asks the same search with one field changed.
//! The content browser's controls: what to look at, and what a card does.
//!
//! Two of these deserve a word. The content type is a filter and not three
//! separate screens, because a player looking for "something that makes it
//! prettier" does not know in advance whether that is a mod, a pack or a
//! shader. The mode — install or suggest — is a stance rather than a per-card
//! button: either they are kitting out their own client or telling staff what
//! the build is missing, and putting both buttons on every card makes every
//! card ask a question that was answered once.

use super::common::Cx;
use crate::components::segment;
use crate::state::LauncherUI;
use gpui::{div, prelude::*, px, AnyElement, ClickEvent, SharedString};
use i18n::t;
use schema::personal::ContentKind;
use uuid::Uuid;

/// Browse / Installed, content type, and the mode switch.
pub fn toolbar(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    // One row, not three: tabs, content types and the mode caption used to take
    // a third of the screen above the list people come here for.
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(segment(
            "content-tab-browse",
            Some("search"),
            t("content-tab-browse"),
            !ui.content_show_installed,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                this.content_show_installed = false;
                cx.notify();
            }),
        ))
        .child(segment(
            "content-tab-installed",
            Some("box"),
            t("content-tab-installed"),
            ui.content_show_installed,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                this.content_show_installed = true;
                this.backend
                    .send(bridge::MessageToBackend::RequestPersonalContent { server_id });
                cx.notify();
            }),
        ))
        .child(div().flex_1())
        // An install or removal is on its way: say so, since the list only
        // changes once the master answers.
        .when(ui.content_busy, |d| {
            d.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .text_size(px(11.))
                    .text_color(gpui::rgb(crate::theme::TEXT_MUTED))
                    .child(crate::icons::ic("hourglass", 14., crate::theme::TEXT_MUTED))
                    .child(t("content-working")),
            )
        })
        .when(!ui.content_show_installed, |d| {
            d.child(kind_row(ui, server_id, cx))
        })
        .into_any_element()
}

fn kind_row(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let kinds = [
        (ContentKind::Mod, "puzzle", "content-kind-mods"),
        (
            ContentKind::ResourcePack,
            "palette",
            "content-kind-resourcepacks",
        ),
        (ContentKind::Shader, "sparkles", "content-kind-shaders"),
    ];

    let chips: Vec<AnyElement> = kinds
        .into_iter()
        .map(|(kind, icon, key)| kind_chip(ui, server_id, kind, icon, key, cx))
        .collect();

    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .children(chips)
        .into_any_element()
}

fn kind_chip(
    ui: &LauncherUI,
    server_id: Uuid,
    kind: ContentKind,
    icon: &'static str,
    key: &'static str,
    cx: &mut Cx,
) -> AnyElement {
    segment(
        SharedString::from(format!("content-kind-{}", kind.as_str())),
        Some(icon),
        t(key),
        ui.content_kind == kind,
        cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.content_kind = kind;
            this.mod_catalog_hits.clear();
            this.search_content(server_id, 0);
            cx.notify();
        }),
    )
}
