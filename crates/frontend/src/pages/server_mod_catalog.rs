//! The full mod catalog screen and the mod detail page.
use super::common::{tabs, Cx};
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::MessageToBackend;
use gpui::{div, prelude::*, px, rgb, rgba, uniform_list, AnyElement, ClickEvent, FontWeight};
use i18n::t;
use uuid::Uuid;

pub fn page(ui: &mut LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    // Kick off a default search on first entry. This runs on every frame, so
    // Один запрос на вход, а не «пока список пуст». Это условие проверяется на
    // каждом кадре: с проверкой по содержимому пустая выдача и ещё не пришедший
    // ответ давали по запросу на кадр.
    if ui.content_requested_for != Some(server_id) {
        ui.search_content(server_id, 0);
    }
    // The installed set is what the catalogue marks cards against, so it is
    // fetched once on entry rather than only when the tab is opened.
    if !ui.personal_content.contains_key(&server_id) {
        ui.backend
            .send(MessageToBackend::RequestPersonalContent { server_id });
        ui.personal_content.insert(server_id, Vec::new());
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

    let picker = super::content_versions::dialog(ui, server_id, cx);
    let toolbar = super::content_toolbar::toolbar(ui, server_id, cx);
    let error = ui.content_error.clone();

    div()
        .size_full()
        .relative()
        .bg(rgb(CONTENT_FALLBACK))
        .child(tabs(ui, cx))
        .child(
            div()
                .absolute()
                .top(px(92.))
                .left(px(32.))
                .right(px(32.))
                .bottom(px(32.))
                .flex()
                .flex_col()
                .min_h_0()
                .gap(px(16.))
                .child(page_header(ui, cx))
                .child(toolbar)
                .children(error.map(refusal))
                .child(if ui.content_show_installed {
                    super::content_installed::view(ui, server_id, cx)
                } else if let Some(selected) = ui.mod_catalog_selected.clone() {
                    super::mod_detail::view(ui, server_id, selected, cx)
                } else {
                    mod_catalog_grid(ui, server_id, cx)
                }),
        )
        .children(picker)
        .into_any_element()
}

/// The master's own wording for a refusal.
///
/// Its text, not ours: "staff blocked this mod" and "the author forbids
/// downloads outside CurseForge" are different problems, and one generic
/// "could not install" for both sends people to support for neither.
fn refusal(message: String) -> AnyElement {
    div()
        .p(px(10.))
        .rounded(px(R_SM))
        .bg(rgba((ERROR << 8) | 0x14))
        .border_1()
        .border_color(rgb(ERROR))
        .flex()
        .items_center()
        .gap(px(8.))
        .child(ic("circle-alert", 14., ERROR))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(11.))
                .text_color(rgb(TEXT_PRIMARY))
                .child(message),
        )
        .into_any_element()
}

/// Only shown with a mod's page open: a way back and what is open.
///
/// The screen had a heading of its own — "CONTENT · 1.21.1 · neoforge" plus a
/// button back to the build's mods. Both said what the tab strip above already
/// says, and the list started a third of the way down the window.
fn page_header(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let Some(selected) = ui.mod_catalog_selected.clone() else {
        return div().into_any_element();
    };

    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(
            div()
                .id("catalog-back")
                .size(px(30.))
                .rounded(px(R_SM))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|d| d.bg(rgba(0xffffff12)))
                .child(ic("arrow-left", 15., TEXT_MUTED))
                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.mod_catalog_selected = None;
                    this.mod_project = None;
                    this.mod_detail_gallery = false;
                    cx.notify();
                })),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(16.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .truncate()
                .child(selected.title),
        )
        .into_any_element()
}

fn mod_catalog_grid(ui: &mut LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    // Пустоту запоминаем отдельно, а сам список читаем ниже по ссылке: копия
    // выдачи снималась на каждом кадре.
    let no_hits = ui.mod_catalog_hits.is_empty();
    let search = super::content_search::search_bar(ui, server_id, cx);
    let pagination = super::content_search::pagination_controls(ui, server_id, cx);

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(search)
        .child(
            div()
                .id("catalog-hits-scroll")
                .flex_1()
                .min_h_0()
                .rounded(px(R_MD))
                .bg(rgb(BG_PANEL))
                .border_1()
                .border_color(rgb(BORDER))
                .p(px(16.))
                .child(if no_hits {
                    // No results and a failed request both end up here; without
                    // the error text both read as a search that never finishes.
                    let (text, color) = match &ui.mod_catalog_error {
                        Some(e) => {
                            let mut args = i18n::FluentArgs::new();
                            args.set("reason", e.clone());
                            (i18n::t_args("content-unavailable", &args), ERROR)
                        }
                        None if ui.content_searching => (t("content-searching"), TEXT_MUTED),
                        // A finished search with nothing in it used to keep
                        // saying "searching" forever.
                        None => (t("content-no-results"), TEXT_MUTED),
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
                    // Виртуальный список: строятся только те карточки, что
                    // видно. Раньше двадцать карточек собирались на каждом
                    // кадре целиком, включая те, что за краем окна.
                    uniform_list(
                        "catalog-hits",
                        ui.mod_catalog_hits.len(),
                        cx.processor(move |this, range: std::ops::Range<usize>, _w, cx| {
                            range
                                .filter_map(|i| this.mod_catalog_hits.get(i).cloned())
                                .map(|hit| super::content_card::mod_card(this, hit, server_id, cx))
                                .collect::<Vec<_>>()
                        }),
                    )
                    .size_full()
                    .into_any_element()
                }),
        )
        .child(pagination)
        .into_any_element()
}
