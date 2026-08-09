//! Экран полного каталога модов и детальной страницы мода в лаунчере.
use super::common::{tabs, Cx};
use crate::components::btn;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::{CatalogHitInfo, MessageToBackend};
use gpui::{
    div, img, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, ObjectFit,
    SharedString,
};
use uuid::Uuid;

pub fn page(ui: &mut LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let server = ui.servers.iter().find(|s| s.id == server_id);
    let mc_ver = server.map(|s| s.mc_version.clone());
    let loader = server.map(|s| s.modloader.as_str().to_string());

    // Автоматически запускаем базовый поиск при первом входе, если выдача пустая
    if ui.mod_catalog_hits.is_empty() {
        ui.backend.send(MessageToBackend::SearchCatalog {
            query: "".to_string(),
            provider: ui.mod_catalog_provider.clone(),
            mc_version: mc_ver.clone(),
            loader: loader.clone(),
        });
    }

    // Автоматически подгружаем иконки через безопасный reqwest loader
    let icon_urls: Vec<String> = ui
        .mod_catalog_hits
        .iter()
        .filter_map(|h| h.icon_url.clone())
        .chain(ui.mod_catalog_selected.iter().filter_map(|s| s.icon_url.clone()))
        .collect();
    for url in icon_urls {
        ui.ensure_optional_mod_icon_loaded(Some(url), cx);
    }

    div()
        .size_full()
        .relative()
        .bg(rgb(CONTENT_FALLBACK))
        .child(tabs(ui, cx))
        .child(
            div()
                .absolute()
                .top(px(104.))
                .left(px(32.))
                .right(px(32.))
                .bottom(px(32.))
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(page_header(ui, server_id, cx))
                .child(if let Some(selected) = ui.mod_catalog_selected.clone() {
                    mod_detail_view(ui, server_id, selected, cx)
                } else {
                    mod_catalog_grid(ui, server_id, cx)
                }),
        )
        .into_any_element()
}

fn page_header(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let server = ui.servers.iter().find(|s| s.id == server_id);
    let subtitle = if let Some(s) = server {
        format!("{} · {}", s.mc_version, s.modloader.as_str())
    } else {
        "Mod Catalog".to_string()
    };

    let title = if let Some(ref selected) = ui.mod_catalog_selected {
        selected.title.clone()
    } else {
        format!("Mod Catalog ({subtitle})")
    };

    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(ic("search", 20., ACCENT))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(18.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(title),
        )
        .child(div().flex_1())
        .when(ui.mod_catalog_selected.is_some(), |d| {
            d.child(btn(
                "back-to-grid-btn",
                "Back to Results",
                false,
                cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    this.mod_catalog_selected = None;
                    cx.notify();
                }),
            ))
        })
        .child(btn(
            "back-to-mods-btn",
            "Back to Server Mods",
            false,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                this.page = crate::state::Page::ServerMods(server_id);
                cx.notify();
            }),
        ))
        .into_any_element()
}

fn mod_catalog_grid(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let hits = ui.mod_catalog_hits.clone();

    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(search_bar(ui, server_id, cx))
        .child(
            div()
                .id("catalog-hits-scroll")
                .flex_1()
                .overflow_y_scroll()
                .rounded(px(R_MD))
                .bg(rgb(BG_PANEL))
                .border_1()
                .border_color(rgb(BORDER))
                .p(px(16.))
                .child(if hits.is_empty() {
                    div()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(14.))
                        .text_color(rgb(TEXT_MUTED))
                        .child("Searching compatible mods...")
                        .into_any_element()
                } else {
                    let items: Vec<AnyElement> = hits
                        .into_iter()
                        .map(|hit| mod_card(ui, hit, server_id, cx))
                        .collect();
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .children(items)
                        .into_any_element()
                }),
        )
        .into_any_element()
}

fn search_bar(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let provider = ui.mod_catalog_provider.clone();
    let server = ui.servers.iter().find(|s| s.id == server_id);
    let mc_ver = server.map(|s| s.mc_version.clone());
    let loader = server.map(|s| s.modloader.as_str().to_string());

    let mc_for_modrinth = mc_ver.clone();
    let ldr_for_modrinth = loader.clone();
    let mc_for_curse = mc_ver.clone();
    let ldr_for_curse = loader.clone();

    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(
            div()
                .flex_1()
                .h(px(40.))
                .px(px(16.))
                .rounded(px(R_SM))
                .bg(rgb(BG_PANEL))
                .border_1()
                .border_color(rgb(BORDER))
                .flex()
                .items_center()
                .gap(px(8.))
                .child(ic("search", 16., TEXT_MUTED))
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(14.))
                        .text_color(rgb(TEXT_MUTED))
                        .child("Search catalog..."),
                ),
        )
        .child(
            div()
                .flex()
                .gap(px(4.))
                .child(btn(
                    "provider-modrinth",
                    "Modrinth",
                    provider == "modrinth",
                    cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                        this.mod_catalog_provider = "modrinth".to_string();
                        this.backend.send(MessageToBackend::SearchCatalog {
                            query: "".to_string(),
                            provider: "modrinth".to_string(),
                            mc_version: mc_for_modrinth.clone(),
                            loader: ldr_for_modrinth.clone(),
                        });
                        cx.notify();
                    }),
                ))
                .child(btn(
                    "provider-curseforge",
                    "CurseForge",
                    provider == "curseforge",
                    cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                        this.mod_catalog_provider = "curseforge".to_string();
                        this.backend.send(MessageToBackend::SearchCatalog {
                            query: "".to_string(),
                            provider: "curseforge".to_string(),
                            mc_version: mc_for_curse.clone(),
                            loader: ldr_for_curse.clone(),
                        });
                        cx.notify();
                    }),
                )),
        )
        .into_any_element()
}

fn mod_card(ui: &LauncherUI, hit: CatalogHitInfo, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let hit_clone = hit.clone();
    let hit_for_req = hit.clone();
    let project_id_str = hit.project_id.clone();

    div()
        .id(SharedString::from(format!("mod-card-{project_id_str}")))
        .h(px(72.))
        .px(px(16.))
        .rounded(px(R_SM))
        .bg(rgba(0xffffff0a))
        .border_1()
        .border_color(rgb(BORDER))
        .hover(|s| s.bg(rgba(0xffffff15)))
        .cursor_pointer()
        .flex()
        .items_center()
        .gap(px(16.))
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.mod_catalog_selected = Some(hit_clone.clone());
            cx.notify();
        }))
        .child(mod_avatar(ui, &hit.icon_url))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .font_family(FONT_PIXEL_ALT)
                                .text_size(px(14.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(TEXT_PRIMARY))
                                .child(hit.title.clone()),
                        )
                        .child(
                            div()
                                .flex_shrink_0()
                                .px(px(6.))
                                .py(px(1.))
                                .rounded(px(R_SM))
                                .bg(rgba(0x0f203688))
                                .font_family(FONT_PIXEL_ALT)
                                .text_size(px(9.))
                                .text_color(rgb(CTA))
                                .child(hit.provider.to_uppercase()),
                        ),
                )
                .child(
                    div()
                        .truncate()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(11.))
                        .text_color(rgb(TEXT_MUTED))
                        .child(hit.description.clone()),
                ),
        )
        .child(btn(
            SharedString::from(format!("btn-req-{project_id_str}")),
            "Request",
            true,
            cx.listener(move |this, _e: &ClickEvent, _w, _cx| {
                this.backend.send(MessageToBackend::SuggestOptionalMod {
                    server_id,
                    build_id: None,
                    provider: hit_for_req.provider.clone(),
                    project_id: hit_for_req.project_id.clone(),
                    title: hit_for_req.title.clone(),
                    icon_url: hit_for_req.icon_url.clone(),
                    description: Some(hit_for_req.description.clone()),
                });
            }),
        ))
        .into_any_element()
}

fn mod_avatar(ui: &LauncherUI, icon_url: &Option<String>) -> AnyElement {
    let outer = div()
        .size(px(44.))
        .rounded(px(R_SM))
        .overflow_hidden()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center();

    if let Some(ref url) = icon_url {
        if let Some(img_data) = ui.optional_mod_icons.get(url).cloned() {
            return outer
                .child(img(img_data).size_full().object_fit(ObjectFit::Cover))
                .into_any_element();
        }
    }
    outer
        .bg(rgba(0xffffff15))
        .border_1()
        .border_color(rgb(BORDER))
        .child(ic("box", 20., TEXT_MUTED))
        .into_any_element()
}

fn mod_detail_view(
    ui: &LauncherUI,
    server_id: Uuid,
    hit: CatalogHitInfo,
    cx: &mut Cx,
) -> AnyElement {
    let hit_for_req = hit.clone();
    div()
        .flex_1()
        .rounded(px(R_MD))
        .bg(rgb(BG_PANEL))
        .border_1()
        .border_color(rgb(BORDER))
        .p(px(32.))
        .flex()
        .flex_col()
        .gap(px(24.))
        .child(
            div()
                .flex()
                .items_start()
                .gap(px(24.))
                .child(mod_avatar(ui, &hit.icon_url))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(
                            div()
                                .font_family(FONT_PIXEL_ALT)
                                .text_size(px(22.))
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
                .child(btn(
                    "request-detail-btn",
                    "Request Mod for Assembly",
                    true,
                    cx.listener(move |this, _e: &ClickEvent, _w, _cx| {
                        this.backend.send(MessageToBackend::SuggestOptionalMod {
                            server_id,
                            build_id: None,
                            provider: hit_for_req.provider.clone(),
                            project_id: hit_for_req.project_id.clone(),
                            title: hit_for_req.title.clone(),
                            icon_url: hit_for_req.icon_url.clone(),
                            description: Some(hit_for_req.description.clone()),
                        });
                    }),
                )),
        )
        .child(
            div()
                .border_t_1()
                .border_color(rgb(BORDER))
                .pt(px(20.))
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(14.))
                .text_color(rgb(TEXT_PRIMARY))
                .line_height(px(22.))
                .child(hit.description),
        )
        .into_any_element()
}
