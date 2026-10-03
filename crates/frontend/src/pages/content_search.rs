// Over 150 lines: the search field, the sort and the paging are one control
// surface — each of them re-asks the same search with one field changed.
//! The catalogue's search row and paging.
//!
//! Every control here ends in `LauncherUI::search_content`. The filters live in
//! the state; a control changes one and re-runs the search. Each of them used
//! to build the message itself, which meant a new filter had to be remembered
//! in eight places.

use super::common::Cx;
use crate::components::segment;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent};
use i18n::t;
use uuid::Uuid;

pub(super) fn pagination_controls(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let total = ui.mod_catalog_total;
    let offset = ui.mod_catalog_offset;
    let limit = if ui.mod_catalog_limit == 0 {
        20
    } else {
        ui.mod_catalog_limit
    };

    let page_now = (offset / limit) + 1;
    let pages = if total == 0 { 1 } else { total.div_ceil(limit) };
    let has_prev = offset >= limit;
    let has_next = offset + limit < total;

    // Одной группой по центру, а не тремя блоками по краям экрана: две подписи
    // в разных углах и число между ними читались как три разных элемента.
    div()
        .flex()
        .justify_center()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(2.))
                .p(px(2.))
                .rounded(px(R_SM))
                .bg(rgba(0x00000040))
                .child(step(
                    "page-prev",
                    "chevron-left",
                    has_prev,
                    offset.saturating_sub(limit),
                    server_id,
                    cx,
                ))
                .child(
                    div()
                        .px(px(12.))
                        .min_w(px(72.))
                        .flex()
                        .justify_center()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(11.))
                        .text_color(rgb(TEXT_SECONDARY))
                        .child(format!("{page_now} / {pages}")),
                )
                .child(step(
                    "page-next",
                    "chevron-right",
                    has_next,
                    offset + limit,
                    server_id,
                    cx,
                )),
        )
        .into_any_element()
}

/// One arrow. Disabled it stays in place and goes dim — a control that
/// disappears at the edges of a list makes the row jump under the cursor.
fn step(
    id: &'static str,
    icon: &'static str,
    enabled: bool,
    target_offset: u32,
    server_id: Uuid,
    cx: &mut Cx,
) -> AnyElement {
    div()
        .id(id)
        .size(px(28.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .justify_center()
        .when(enabled, |d| {
            d.cursor_pointer()
                .hover(|s| s.bg(rgba(0xffffff12)))
                .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    this.search_content(server_id, target_offset);
                    cx.notify();
                }))
        })
        .child(ic(icon, 14., if enabled { TEXT_SECONDARY } else { BORDER }))
        .into_any_element()
}

/// The search field, the sort and the two providers.
///
/// Every control here ends in the same call: the filters live in the state, and
/// each of these changes one of them and re-runs the search. Before that, each
/// one built the message itself, and a new filter meant finding all eight.
pub(super) fn search_bar(ui: &mut LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let provider = ui.mod_catalog_provider.clone();
    let query = ui.mod_catalog_query.clone();
    let focus_handle = ui
        .mod_catalog_focus
        .get_or_insert_with(|| cx.focus_handle())
        .clone();
    let focus_for_click = focus_handle.clone();

    div()
        .flex()
        .items_center()
        .gap(px(8.))
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
                    focus_for_click.focus(window, cx);
                    cx.notify();
                }))
                .on_key_down(
                    cx.listener(move |this, event: &gpui::KeyDownEvent, _w, cx| {
                        if let Some(text) = super::common::pasted(event, cx) {
                            this.mod_catalog_query.push_str(&text);
                            cx.notify();
                            return;
                        }
                        match event.keystroke.key.as_str() {
                            "backspace" => {
                                this.mod_catalog_query.pop();
                            }
                            "enter" => this.search_content(server_id, 0),
                            "space" => this.mod_catalog_query.push(' '),
                            // `key_char` already accounts for shift and layout,
                            // and is empty under cmd/ctrl, so shortcuts don't
                            // end up typed into the query.
                            _ => {
                                if let Some(ch) = event.keystroke.key_char.as_deref() {
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
                            t("content-search-placeholder")
                        } else {
                            format!("{query}_")
                        }),
                )
                .when(!query.is_empty(), |d| {
                    d.child(
                        div()
                            .id("catalog-search-clear")
                            .cursor_pointer()
                            .child(ic("x", 13., TEXT_MUTED))
                            .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                                this.mod_catalog_query.clear();
                                this.search_content(server_id, 0);
                                cx.notify();
                            })),
                    )
                }),
        )
        .child(sort_button(ui, server_id, cx))
        .child(
            // Два переключателя подряд, без общей подложки: подложка добавляла
            // третью рамку вокруг того, что и так обведено.
            div()
                .h(px(40.))
                .flex()
                .items_center()
                .gap(px(4.))
                .child(provider_button(
                    "provider-modrinth",
                    "Modrinth",
                    "modrinth",
                    provider == "modrinth",
                    server_id,
                    cx,
                ))
                .child(provider_button(
                    "provider-curseforge",
                    "CurseForge",
                    "curseforge",
                    provider == "curseforge",
                    server_id,
                    cx,
                )),
        )
        .into_any_element()
}

fn provider_button(
    id: &'static str,
    label: &'static str,
    value: &'static str,
    active: bool,
    server_id: Uuid,
    cx: &mut Cx,
) -> AnyElement {
    segment(
        id,
        None,
        label,
        active,
        cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.mod_catalog_provider = value.to_string();
            this.search_content(server_id, 0);
            cx.notify();
        }),
    )
}

/// Translation key for a sort. The provider's own words — `follows`, `updated`
/// — are not a label for a person.
fn sort_key(sort: &str) -> &'static str {
    match sort {
        "downloads" => "content-sort-downloads",
        "follows" => "content-sort-follows",
        "newest" => "content-sort-newest",
        "updated" => "content-sort-updated",
        _ => "content-sort-relevance",
    }
}

fn sort_button(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    const SORTS: [&str; 5] = ["relevance", "downloads", "follows", "newest", "updated"];
    let current = ui.content_sort.clone();
    let next = SORTS
        .iter()
        .position(|s| *s == current)
        .map(|i| SORTS[(i + 1) % SORTS.len()])
        .unwrap_or(SORTS[0]);

    div()
        .id("catalog-sort")
        .h(px(40.))
        .px(px(12.))
        .rounded(px(R_SM))
        .bg(rgb(BG_PANEL))
        .border_1()
        .border_color(rgb(BORDER))
        .flex()
        .items_center()
        .gap(px(6.))
        .cursor_pointer()
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .child(ic("sort", 14., TEXT_MUTED))
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(t(sort_key(&current))),
        )
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.content_sort = next.to_string();
            this.search_content(server_id, 0);
            cx.notify();
        }))
        .into_any_element()
}
