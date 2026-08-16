//! Страница мода: описание в markdown, скриншоты, метаданные.
//!
//! Выдача поиска несёт только заголовок и одну строку описания, поэтому полная
//! страница тянется отдельным запросом и приезжает в `ui.mod_project`. Пока она
//! в пути, показываем короткое описание из выдачи — экран не должен быть пустым.

use super::common::Cx;
use super::mod_detail_body::description;
use super::mod_detail_parts::{gallery, tab_button};
use crate::components::btn;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::{CatalogHitInfo, MessageToBackend};
use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent, FontWeight};
use uuid::Uuid;

pub fn view(ui: &mut LauncherUI, server_id: Uuid, hit: CatalogHitInfo, cx: &mut Cx) -> AnyElement {
    let project = ui.mod_project.clone();
    let shots = project
        .as_ref()
        .map(|p| p.gallery.clone())
        .unwrap_or_default();
    for url in &shots {
        ui.ensure_optional_mod_icon_loaded(Some(url.clone()), cx);
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

fn header(ui: &LauncherUI, server_id: Uuid, hit: &CatalogHitInfo, cx: &mut Cx) -> AnyElement {
    let for_request = hit.clone();
    let is_installed = super::mod_icon::is_mod_installed(ui, server_id, &hit.title);
    let is_pending = ui.suggested_mods.contains(&hit.project_id);

    let action_btn: AnyElement = if is_installed {
        crate::components::badge("Installed", CTA).into_any_element()
    } else if is_pending {
        crate::components::badge("Pending", WARNING).into_any_element()
    } else {
        btn(
            "request-detail-btn",
            "Suggest Mod for Assembly",
            true,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                let build_id = this.server(&server_id).and_then(|s| s.current_build_id);
                this.suggested_mods.insert(for_request.project_id.clone());
                this.backend.send(MessageToBackend::SuggestOptionalMod {
                    server_id,
                    build_id,
                    provider: for_request.provider.clone(),
                    project_id: for_request.project_id.clone(),
                    title: for_request.title.clone(),
                    icon_url: for_request.icon_url.clone(),
                    description: Some(for_request.description.clone()),
                });
                cx.notify();
            }),
        )
        .into_any_element()
    };

    div()
        .flex()
        .items_start()
        .gap(px(24.))
        .child(super::server_mod_catalog::mod_avatar(ui, &hit.icon_url))
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
                        .text_size(if hit.title.len() > 45 {
                            px(15.)
                        } else if hit.title.len() > 25 {
                            px(18.)
                        } else {
                            px(22.)
                        })
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(CTA))
                        .child(hit.title.clone()),
                )
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(12.))
                        .text_color(rgb(TEXT_MUTED))
                        .child(format!(
                            "Provider: {} | Downloads: {}",
                            hit.provider.to_uppercase(),
                            hit.downloads
                        )),
                ),
        )
        .child(action_btn)
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
            "Description",
            !gallery_active,
            cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.mod_detail_gallery = false;
                cx.notify();
            }),
        ))
        .when(shots > 0, |d| {
            d.child(tab_button(
                "mod-tab-gallery",
                &format!("Gallery ({shots})"),
                gallery_active,
                cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.mod_detail_gallery = true;
                    cx.notify();
                }),
            ))
        })
        .into_any_element()
}
