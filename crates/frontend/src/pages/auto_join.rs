// Over 150 lines: a checkbox for a build with one server, a closed select and
// its open list for a build with several.
//! Whether the game joins a server once it has started, and which one.
//!
//! A build with one entry point gets a checkbox: there is nothing to choose
//! but yes or no. With several the choice is which, so it is a select whose
//! first line is "don't join". Entry points are what the player may connect to
//! directly — the proxy when the build has one, every game server otherwise.
//!
//! The list opens as a dialog, like the Java picker, for the same reason: GPUI
//! paints in tree order, and a list inside its own row would go under the rows
//! that follow it.

use super::common::Cx;
use crate::components::checkbox_row;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::AutoJoin;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use i18n::t;
use uuid::Uuid;

/// The game servers of the build a player may join directly, with their names.
fn entry_points(ui: &LauncherUI, server_id: Uuid) -> Vec<(Uuid, String)> {
    let Some(server) = ui.server(&server_id) else {
        return Vec::new();
    };
    let has_proxy = server.game_servers.iter().any(|node| node.proxy);
    server
        .game_servers
        .iter()
        .filter(|node| !has_proxy || node.proxy)
        .map(|node| (node.id, node.name.clone()))
        .collect()
}

/// The entry point in force: the one picked if it is still there, the first one
/// otherwise; `None` when auto-join is off.
fn joined(ui: &LauncherUI, server_id: Uuid, entries: &[(Uuid, String)]) -> Option<Uuid> {
    match ui.config.auto_join.get(&server_id).copied() {
        Some(AutoJoin::Off) => None,
        Some(AutoJoin::To(id)) if entries.iter().any(|(e, _)| *e == id) => Some(id),
        _ => entries.first().map(|(id, _)| *id),
    }
}

/// The control that goes into the settings row.
pub fn control(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let entries = entry_points(ui, server_id);
    let has_address = ui.server(&server_id).is_some_and(|s| s.mc_host.is_some());
    if entries.is_empty() && !has_address {
        return div()
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(12.))
            .text_color(rgb(TEXT_MUTED))
            .child(t("settings-autojoin-none"))
            .into_any_element();
    }
    if entries.len() <= 1 {
        checkbox(ui, server_id, cx)
    } else {
        closed(ui, server_id, &entries, cx)
    }
}

fn checkbox(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let enabled = ui.config.auto_join.get(&server_id) != Some(&AutoJoin::Off);
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(checkbox_row(
            SharedString::from(format!("cs-autojoin-{server_id}")),
            enabled,
            true,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                let on = this.config.auto_join.get(&server_id) != Some(&AutoJoin::Off);
                this.set_server_auto_join(server_id, if on { Some(AutoJoin::Off) } else { None });
                cx.notify();
            }),
        ))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(t("settings-autojoin-show")),
        )
        .into_any_element()
}

fn closed(ui: &LauncherUI, server_id: Uuid, entries: &[(Uuid, String)], cx: &mut Cx) -> AnyElement {
    let current = joined(ui, server_id, entries);
    let text = current
        .and_then(|id| entries.iter().find(|(e, _)| *e == id))
        .map(|(_, name)| name.clone())
        .unwrap_or_else(|| t("settings-autojoin-off"));

    div()
        .id("autojoin-select")
        .w(px(280.))
        .h(px(36.))
        .px(px(12.))
        .rounded(px(R_SM))
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(if ui.auto_join_picker_open {
            ACCENT
        } else {
            BORDER
        }))
        .flex()
        .items_center()
        .gap(px(8.))
        .cursor_pointer()
        .hover(|d| d.border_color(rgb(CTA)))
        .child(ic("server", 14., TEXT_MUTED))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(12.))
                .text_color(rgb(if current.is_some() {
                    CTA
                } else {
                    TEXT_SECONDARY
                }))
                .truncate()
                .child(text),
        )
        .child(ic(
            if ui.auto_join_picker_open {
                "chevron-up"
            } else {
                "chevron-down"
            },
            13.,
            TEXT_MUTED,
        ))
        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
            this.auto_join_picker_open = !this.auto_join_picker_open;
            cx.notify();
        }))
        .into_any_element()
}

/// The open list, drawn at page level so nothing can cover it.
pub fn dialog(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> Option<AnyElement> {
    if !ui.auto_join_picker_open {
        return None;
    }
    let entries = entry_points(ui, server_id);
    let current = joined(ui, server_id, &entries);

    let mut rows: Vec<AnyElement> = vec![row(
        "autojoin-opt-off".to_string(),
        t("settings-autojoin-off"),
        current.is_none(),
        server_id,
        AutoJoin::Off,
        cx,
    )];
    rows.extend(entries.iter().map(|(id, name)| {
        row(
            format!("autojoin-opt-{id}"),
            name.clone(),
            current == Some(*id),
            server_id,
            AutoJoin::To(*id),
            cx,
        )
    }));

    Some(
        div()
            .id("autojoin-backdrop")
            .absolute()
            .inset_0()
            .bg(rgba(0x000000aa))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.auto_join_picker_open = false;
                cx.notify();
            }))
            .child(
                div()
                    .id("autojoin-dialog")
                    .occlude()
                    .on_click(|_, _, _| {})
                    .w(px(420.))
                    .max_h(px(420.))
                    .flex()
                    .flex_col()
                    .rounded(px(R_LG))
                    .bg(rgb(BG_PANEL))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .child(header(cx))
                    .child(
                        div()
                            .id("autojoin-options")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .p(px(8.))
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .children(rows),
                    ),
            )
            .into_any_element(),
    )
}

fn header(cx: &mut Cx) -> AnyElement {
    div()
        .h(px(48.))
        .px(px(16.))
        .flex()
        .items_center()
        .gap(px(8.))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(ic("server", 15., ACCENT))
        .child(
            div()
                .flex_1()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(13.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(t("settings-autojoin-title")),
        )
        .child(
            div()
                .id("autojoin-close")
                .size(px(28.))
                .rounded(px(R_SM))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|d| d.bg(rgba(0xffffff10)))
                .child(ic("x", 14., TEXT_MUTED))
                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.auto_join_picker_open = false;
                    cx.notify();
                })),
        )
        .into_any_element()
}

fn row(
    id: String,
    text: String,
    active: bool,
    server_id: Uuid,
    choice: AutoJoin,
    cx: &mut Cx,
) -> AnyElement {
    div()
        .id(SharedString::from(id))
        .px(px(10.))
        .h(px(32.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .gap(px(8.))
        .cursor_pointer()
        .bg(if active {
            rgba((CTA << 8) | 0x16)
        } else {
            rgba(0x00000000)
        })
        .hover(|d| d.bg(rgba(0xffffff10)))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(12.))
                .font_weight(if active {
                    FontWeight::BOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(rgb(if active { CTA } else { TEXT_SECONDARY }))
                .truncate()
                .child(text),
        )
        .when(active, |d| d.child(ic("check", 13., CTA)))
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.auto_join_picker_open = false;
            this.set_server_auto_join(server_id, Some(choice));
            cx.notify();
        }))
        .into_any_element()
}
