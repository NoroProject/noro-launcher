//! The install button on a mod's own page, wider than the one on its catalogue card.

use super::common::Cx;
use crate::icons::ic;
use crate::state::{ContentMode, LauncherUI};
use crate::theme::*;
use bridge::{CatalogHitInfo, MessageToBackend};
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight};
use i18n::t;
use uuid::Uuid;

/// The action as the mod's own page shows it: icon plus words.
///
/// A card has room for a glyph and twenty rows of them; a page has one action
/// and space to name it.
pub(super) fn detail_action(
    ui: &LauncherUI,
    server_id: Uuid,
    hit: &CatalogHitInfo,
    cx: &mut Cx,
) -> AnyElement {
    let project_id = hit.project_id.clone();
    let installed = super::mod_icon::is_mod_installed(ui, server_id, &hit.title)
        || ui
            .personal_content
            .get(&server_id)
            .is_some_and(|items| items.iter().any(|i| i.content.project_id == project_id));

    if installed {
        return wide_state("check", t("mods-installed"), SUCCESS);
    }
    if ui.content_mode == ContentMode::Suggest && ui.suggested_mods.contains(&project_id) {
        return wide_state("hourglass", t("content-pending"), WARNING);
    }

    let hit = hit.clone();
    let suggest = ui.content_mode == ContentMode::Suggest;
    let (icon, label) = if suggest {
        ("send", t("content-mode-suggest"))
    } else {
        ("plus", t("content-install"))
    };

    div()
        .id("mod-detail-action")
        .flex_shrink_0()
        .h(px(40.))
        .px(px(18.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .gap(px(8.))
        .cursor_pointer()
        .bg(rgb(CTA))
        .text_color(rgb(ON_CTA))
        .hover(|d| d.bg(rgb(CTA_HOV)))
        .child(ic(icon, 15., ON_CTA))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(13.))
                .font_weight(FontWeight::BOLD)
                .child(label),
        )
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            if suggest {
                let build_id = this.server(&server_id).and_then(|s| s.current_build_id);
                this.suggested_mods.insert(hit.project_id.clone());
                this.backend.send(MessageToBackend::SuggestOptionalMod {
                    server_id,
                    build_id,
                    provider: hit.provider.clone(),
                    project_id: hit.project_id.clone(),
                    title: hit.title.clone(),
                    icon_url: hit.icon_url.clone(),
                    description: Some(hit.description.clone()),
                });
            } else {
                this.content_error = None;
                this.content_picker = Some((hit.provider.clone(), hit.project_id.clone()));
                this.content_versions
                    .remove(&(hit.provider.clone(), hit.project_id.clone()));
                this.backend.send(MessageToBackend::RequestContentVersions {
                    provider: hit.provider.clone(),
                    project_id: hit.project_id.clone(),
                    server_id,
                });
            }
            cx.notify();
        }))
        .into_any_element()
}

/// Nothing to press, said in words.
fn wide_state(icon: &'static str, label: String, colour: u32) -> AnyElement {
    div()
        .flex_shrink_0()
        .h(px(40.))
        .px(px(16.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .gap(px(8.))
        .bg(rgba(0x00000040))
        .border_1()
        .border_color(rgb(colour))
        .child(ic(icon, 15., colour))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(colour))
                .child(label),
        )
        .into_any_element()
}
