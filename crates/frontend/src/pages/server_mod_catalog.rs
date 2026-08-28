//! The full mod catalog screen and the mod detail page.
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

    // Kick off a default search on first entry. This runs on every frame, so
    // the error guard matters: without it a catalog that is down would take a
    // request per frame instead of one.
    if ui.mod_catalog_hits.is_empty() && ui.mod_catalog_error.is_none() {
        ui.backend.send(MessageToBackend::SearchCatalog {
            query: "".to_string(),
            provider: ui.mod_catalog_provider.clone(),
            mc_version: mc_ver.clone(),
            loader: loader.clone(),
            offset: 0,
        });
    }

    let icon_urls: Vec<String> = ui
        .mod_catalog_hits
        .iter()
        .filter_map(|h| h.icon_url.clone())
        .chain(
            ui.mod_catalog_selected
                .iter()
                .filter_map(|s| s.icon_url.clone()),
        )
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
                .min_h_0()
                .gap(px(16.))
                .child(page_header(ui, server_id, cx))
                .child(if let Some(selected) = ui.mod_catalog_selected.clone() {
                    super::mod_detail::view(ui, server_id, selected, cx)
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
                .flex_1()
                .min_w_0()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(18.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .truncate()
                .child(title),
        )
        .when(ui.mod_catalog_selected.is_some(), |d| {
            d.child(btn(
                "back-to-grid-btn",
                "Back to Results",
                false,
                cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    this.mod_catalog_selected = None;
                    this.mod_project = None;
                    this.mod_detail_gallery = false;
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

fn mod_catalog_grid(ui: &mut LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let hits = ui.mod_catalog_hits.clone();

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(search_bar(ui, server_id, cx))
        .child(
            div()
                .id("catalog-hits-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .rounded(px(R_MD))
                .bg(rgb(BG_PANEL))
                .border_1()
                .border_color(rgb(BORDER))
                .p(px(16.))
                .child(if hits.is_empty() {
                    // No results and a failed request both end up here; without
                    // the error text both read as a search that never finishes.
                    let (text, color) = match &ui.mod_catalog_error {
                        Some(e) => (format!("Catalog unavailable: {e}"), ERROR),
                        None => ("Searching compatible mods...".to_string(), TEXT_MUTED),
                    };
                    div()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .px(px(16.))
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(14.))
                        .text_color(rgb(color))
                        .child(text)
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
        .child(pagination_controls(ui, server_id, cx))
        .into_any_element()
}

fn pagination_controls(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let total = ui.mod_catalog_total;
    let offset = ui.mod_catalog_offset;
    let limit = if ui.mod_catalog_limit == 0 {
        20
    } else {
        ui.mod_catalog_limit
    };

    let current_page = (offset / limit) + 1;
    let total_pages = if total == 0 { 1 } else { total.div_ceil(limit) };

    let server = ui.servers.iter().find(|s| s.id == server_id);
    let mc_ver = server.map(|s| s.mc_version.clone());
    let loader = server.map(|s| s.modloader.as_str().to_string());
    let provider = ui.mod_catalog_provider.clone();

    let has_prev = offset >= limit;
    let has_next = offset + limit < total;

    let prev_offset = if has_prev { offset - limit } else { 0 };
    let next_offset = offset + limit;

    let mc_prev = mc_ver.clone();
    let ldr_prev = loader.clone();
    let prov_prev = provider.clone();

    let mc_next = mc_ver.clone();
    let ldr_next = loader.clone();
    let prov_next = provider.clone();

    div()
        .flex()
        .items_center()
        .justify_between()
        .px(px(8.))
        .py(px(4.))
        .child(div().flex().items_center().gap(px(8.)).when(has_prev, |d| {
            d.child(btn(
                "prev-page-btn",
                "< Prev",
                false,
                cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    this.mod_catalog_error = None;
                    this.backend.send(MessageToBackend::SearchCatalog {
                        query: "".to_string(),
                        provider: prov_prev.clone(),
                        mc_version: mc_prev.clone(),
                        loader: ldr_prev.clone(),
                        offset: prev_offset,
                    });
                    cx.notify();
                }),
            ))
        }))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(13.))
                .text_color(rgb(TEXT_MUTED))
                .child(format!(
                    "Page {current_page} of {total_pages} ({total} mods)"
                )),
        )
        .child(div().flex().items_center().gap(px(8.)).when(has_next, |d| {
            d.child(btn(
                "next-page-btn",
                "Next >",
                false,
                cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    this.mod_catalog_error = None;
                    this.backend.send(MessageToBackend::SearchCatalog {
                        query: "".to_string(),
                        provider: prov_next.clone(),
                        mc_version: mc_next.clone(),
                        loader: ldr_next.clone(),
                        offset: next_offset,
                    });
                    cx.notify();
                }),
            ))
        }))
        .into_any_element()
}

fn search_bar(ui: &mut LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let provider = ui.mod_catalog_provider.clone();
    let query = ui.mod_catalog_query.clone();
    let focus_handle = ui
        .mod_catalog_focus
        .get_or_insert_with(|| cx.focus_handle())
        .clone();

    let server = ui.servers.iter().find(|s| s.id == server_id);
    let mc_ver = server.map(|s| s.mc_version.clone());
    let loader = server.map(|s| s.modloader.as_str().to_string());

    let mc_for_modrinth = mc_ver.clone();
    let ldr_for_modrinth = loader.clone();
    let mc_for_curse = mc_ver.clone();
    let ldr_for_curse = loader.clone();

    let mc_for_enter = mc_ver.clone();
    let ldr_for_enter = loader.clone();

    let mc_for_submit = mc_ver.clone();
    let ldr_for_submit = loader.clone();

    let focus_handle_click = focus_handle.clone();

    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(
            div()
                .id("catalog-search-input")
                .track_focus(&focus_handle)
                .flex_1()
                .h(px(40.))
                .px(px(16.))
                .rounded(px(R_SM))
                .bg(rgb(BG_PANEL))
                .border_1()
                .border_color(rgb(BORDER))
                .focus(|s| s.border_color(rgb(ACCENT)))
                .flex()
                .items_center()
                .gap(px(8.))
                .cursor_text()
                .on_click(cx.listener(move |_this, _e: &ClickEvent, window, cx| {
                    focus_handle_click.focus(window, cx);
                    cx.notify();
                }))
                .on_key_down(
                    cx.listener(move |this, event: &gpui::KeyDownEvent, _w, cx| {
                        let keystroke = &event.keystroke;
                        match keystroke.key.as_str() {
                            "backspace" => {
                                this.mod_catalog_query.pop();
                            }
                            "enter" => {
                                let q = this.mod_catalog_query.trim().to_string();
                                let prov = this.mod_catalog_provider.clone();
                                let mc = mc_for_enter.clone();
                                let ldr = ldr_for_enter.clone();
                                this.mod_catalog_offset = 0;
                                this.mod_catalog_error = None;
                                this.backend.send(MessageToBackend::SearchCatalog {
                                    query: q,
                                    provider: prov,
                                    mc_version: mc,
                                    loader: ldr,
                                    offset: 0,
                                });
                            }
                            "space" => {
                                this.mod_catalog_query.push(' ');
                            }
                            // `key_char` already accounts for shift and layout,
                            // and is empty under cmd/ctrl, so shortcuts don't
                            // end up typed into the query.
                            _ => {
                                if let Some(ch) = keystroke.key_char.as_deref() {
                                    this.mod_catalog_query.push_str(ch);
                                }
                            }
                        }
                        cx.notify();
                    }),
                )
                .child(ic("search", 16., TEXT_MUTED))
                .child(
                    div()
                        .flex_1()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(14.))
                        .text_color(if query.is_empty() {
                            rgb(TEXT_MUTED)
                        } else {
                            rgb(TEXT_PRIMARY)
                        })
                        .child(if query.is_empty() {
                            "Search catalog...".to_string()
                        } else {
                            format!("{query}_")
                        }),
                )
                .when(!query.is_empty(), |d| {
                    let mc_clear = mc_ver.clone();
                    let ldr_clear = loader.clone();
                    let prov_clear = provider.clone();
                    d.child(
                        div()
                            .id("catalog-search-clear")
                            .cursor_pointer()
                            .font_family(FONT_PIXEL_ALT)
                            .text_size(px(13.))
                            .text_color(rgb(TEXT_MUTED))
                            .hover(|s| s.text_color(rgb(CTA)))
                            .child("✕")
                            .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                                this.mod_catalog_query.clear();
                                this.mod_catalog_offset = 0;
                                this.mod_catalog_error = None;
                                this.backend.send(MessageToBackend::SearchCatalog {
                                    query: "".to_string(),
                                    provider: prov_clear.clone(),
                                    mc_version: mc_clear.clone(),
                                    loader: ldr_clear.clone(),
                                    offset: 0,
                                });
                                cx.notify();
                            })),
                    )
                }),
        )
        .child(btn(
            "search-submit-btn",
            "Search",
            false,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                let q = this.mod_catalog_query.trim().to_string();
                let prov = this.mod_catalog_provider.clone();
                let mc = mc_for_submit.clone();
                let ldr = ldr_for_submit.clone();
                this.mod_catalog_offset = 0;
                this.mod_catalog_error = None;
                this.backend.send(MessageToBackend::SearchCatalog {
                    query: q,
                    provider: prov,
                    mc_version: mc,
                    loader: ldr,
                    offset: 0,
                });
                cx.notify();
            }),
        ))
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
                        let q = this.mod_catalog_query.trim().to_string();
                        this.mod_catalog_error = None;
                        this.backend.send(MessageToBackend::SearchCatalog {
                            query: q,
                            provider: "modrinth".to_string(),
                            mc_version: mc_for_modrinth.clone(),
                            loader: ldr_for_modrinth.clone(),
                            offset: 0,
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
                        let q = this.mod_catalog_query.trim().to_string();
                        this.mod_catalog_error = None;
                        this.backend.send(MessageToBackend::SearchCatalog {
                            query: q,
                            provider: "curseforge".to_string(),
                            mc_version: mc_for_curse.clone(),
                            loader: ldr_for_curse.clone(),
                            offset: 0,
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

    let is_installed = super::mod_icon::is_mod_installed(ui, server_id, &hit.title);
    let is_pending = ui.suggested_mods.contains(&hit.project_id);

    let action_btn: AnyElement = if is_installed {
        crate::components::badge("Installed", CTA).into_any_element()
    } else if is_pending {
        crate::components::badge("Pending", WARNING).into_any_element()
    } else {
        btn(
            SharedString::from(format!("btn-req-{project_id_str}")),
            "Suggest",
            true,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                let build_id = this.server(&server_id).and_then(|s| s.current_build_id);
                this.suggested_mods.insert(hit_for_req.project_id.clone());
                this.backend.send(MessageToBackend::SuggestOptionalMod {
                    server_id,
                    build_id,
                    provider: hit_for_req.provider.clone(),
                    project_id: hit_for_req.project_id.clone(),
                    title: hit_for_req.title.clone(),
                    icon_url: hit_for_req.icon_url.clone(),
                    description: Some(hit_for_req.description.clone()),
                });
                cx.notify();
            }),
        )
        .into_any_element()
    };

    div()
        .id(SharedString::from(format!("mod-card-{project_id_str}")))
        .h(px(72.))
        .px(px(16.))
        .rounded(px(R_SM))
        .bg(rgba(0xffffff0a))
        .border_1()
        .border_color(rgb(BORDER))
        .flex()
        .items_center()
        .gap(px(16.))
        .child(
            div()
                .id(SharedString::from(format!(
                    "mod-card-info-{project_id_str}"
                )))
                .flex_1()
                .min_w_0()
                .flex()
                .items_center()
                .gap(px(16.))
                .cursor_pointer()
                .hover(|s| s.bg(rgba(0xffffff05)))
                .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    this.mod_catalog_selected = Some(hit_clone.clone());
                    this.mod_project = None;
                    this.mod_detail_gallery = false;
                    this.backend.send(MessageToBackend::RequestModProject {
                        provider: hit_clone.provider.clone(),
                        project_id: hit_clone.project_id.clone(),
                    });
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
                                        .truncate()
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
                ),
        )
        .child(action_btn)
        .into_any_element()
}

pub(super) fn mod_avatar(ui: &LauncherUI, icon_url: &Option<String>) -> AnyElement {
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
