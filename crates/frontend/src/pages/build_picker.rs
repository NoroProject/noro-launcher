// Over 150 lines: the block, its open list and a row of it. Split, a row has
// nowhere to say what picking it does.
//! The build block in the bottom bar: which version is about to launch, and,
//! when the server carries more than one, the way to pick another.
//!
//! The block itself is the control. A separate dropdown beside it showed the
//! version twice — the published one in the block, the picked one in the
//! dropdown — and two numbers next to each other read as a contradiction.
//! With a single build there is nothing to pick, and no chevron to suggest it.

use super::common::Cx;
use super::game_bar::{BAR_HEIGHT, BAR_INSET, BAR_PADDING};
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use i18n::t;
use schema::{BuildOption, ServerEntry};
use uuid::Uuid;

pub fn build_block(ui: &LauncherUI, server: &ServerEntry, cx: &mut Cx) -> AnyElement {
    let current = active_build(ui, server);
    // A dash rather than "draft": there is no build at all here, and "draft"
    // reads as "there is one, it's just rough".
    let version = current
        .map(|b| b.version.clone())
        .or_else(|| server.current_version.clone())
        .unwrap_or_else(|| "—".into());
    let preview = current.is_some_and(|b| !b.published);
    let pickable = server.available_builds.len() > 1;
    face(
        &version,
        preview,
        pickable,
        pickable && ui.build_picker_open,
        cx,
    )
}

/// The open list, drawn at page level over a backdrop that closes it — the
/// same as the Java list. Closing on a click outside the block itself didn't
/// work: the list sits outside the block's bounds, so a click on a row counted
/// as outside and shut the list before the row got it.
pub fn build_menu(ui: &LauncherUI, server: &ServerEntry, cx: &mut Cx) -> Option<AnyElement> {
    if !ui.build_picker_open || server.available_builds.len() < 2 {
        return None;
    }
    let active = active_build(ui, server).map(|b| b.id);
    Some(
        div()
            .id("build-picker-backdrop")
            .absolute()
            .inset_0()
            .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.build_picker_open = false;
                cx.notify();
            }))
            .child(menu(server, active, cx))
            .into_any_element(),
    )
}

fn active_build<'a>(ui: &LauncherUI, server: &'a ServerEntry) -> Option<&'a BuildOption> {
    let active = ui
        .selected_build
        .get(&server.id)
        .copied()
        .flatten()
        .or(server.current_build_id);
    server
        .available_builds
        .iter()
        .find(|b| Some(b.id) == active)
}

fn face(version: &str, preview: bool, pickable: bool, open: bool, cx: &mut Cx) -> AnyElement {
    div()
        .id("build-block-face")
        .h(px(56.))
        .pl(px(4.))
        .pr(px(if pickable { 12. } else { 4. }))
        .rounded(px(R_SM))
        .border_1()
        .border_color(if open { rgb(CTA) } else { rgba(0) })
        .flex()
        .items_center()
        .gap(px(12.))
        .when(pickable, |d| {
            d.cursor_pointer()
                .hover(|d| d.bg(rgba(0xffffff0a)).border_color(rgb(BORDER)))
                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.build_picker_open = !this.build_picker_open;
                    cx.notify();
                }))
        })
        .child(
            div()
                .size(px(48.))
                .rounded(px(R_SM))
                .bg(rgb(BG_INPUT))
                .border_1()
                .border_color(rgb(BORDER))
                .flex()
                .items_center()
                .justify_center()
                .child(ic("server", 22., CTA)),
        )
        .child(
            div()
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
                                .font_family(FONT_PIXEL_ALT)
                                .text_size(px(9.))
                                .text_color(rgb(TEXT_MUTED))
                                .font_weight(FontWeight::BOLD)
                                .child(t("game-build")),
                        )
                        .when(preview, |d| d.child(tag(t("game-build-preview"), ACCENT))),
                )
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(20.))
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .text_color(rgb(if preview { ACCENT } else { TEXT_PRIMARY }))
                        .child(version.to_string()),
                ),
        )
        .when(pickable, |d| {
            d.child(ic(
                if open { "chevron-up" } else { "chevron-down" },
                16.,
                if open { CTA } else { TEXT_MUTED },
            ))
        })
        .into_any_element()
}

/// Opens upward, over the build block: the bar already sits against the
/// bottom of the window.
fn menu(server: &ServerEntry, active: Option<Uuid>, cx: &mut Cx) -> AnyElement {
    let rows: Vec<AnyElement> = server
        .available_builds
        .iter()
        .map(|build| row(server, build, active == Some(build.id), cx))
        .collect();

    div()
        .id("build-picker-menu")
        .occlude()
        .on_click(|_, _, _| {})
        .absolute()
        .bottom(px(BAR_INSET + BAR_HEIGHT + 8.))
        .left(px(BAR_INSET + BAR_PADDING))
        .w(px(280.))
        .p(px(8.))
        .rounded(px(R_LG))
        .bg(rgb(BG_PANEL))
        .border_1()
        .border_color(rgb(BORDER))
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .px(px(8.))
                .pt(px(4.))
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(10.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(TEXT_MUTED))
                .child(t("game-build-pick")),
        )
        .child(
            div()
                .id("build-picker-rows")
                // An active server piles up builds; the list scrolls rather
                // than running off the top of the window.
                .max_h(px(288.))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap(px(4.))
                .children(rows),
        )
        .into_any_element()
}

fn row(server: &ServerEntry, build: &BuildOption, active: bool, cx: &mut Cx) -> AnyElement {
    let server_id = server.id;
    let is_current = server.current_build_id == Some(build.id);
    // The published build is picked as `None`, not by id: an id pins the player
    // to it, and the next publish would pass them by.
    let pick = (!is_current).then_some(build.id);

    div()
        .id(("build-option", build.id.as_u128() as u64))
        .h(px(44.))
        .px(px(12.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .flex()
        .items_center()
        .gap(px(8.))
        .bg(if active {
            rgba((CTA << 8) | 0x14)
        } else {
            rgba(0)
        })
        .border_1()
        .border_color(if active {
            rgba((CTA << 8) | 0x55)
        } else {
            rgba(0)
        })
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .child(
            div()
                .flex_1()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(14.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(if active {
                    CTA
                } else if build.published {
                    TEXT_PRIMARY
                } else {
                    ACCENT
                }))
                .child(build.version.clone()),
        )
        .when(is_current, |d| {
            d.child(tag(t("game-build-current"), SUCCESS))
        })
        // A preview carries its tag whether picked or not: the player should
        // know before launching, not after.
        .when(!build.published, |d| {
            d.child(tag(t("game-build-preview"), ACCENT))
        })
        .child(
            div()
                .w(px(16.))
                .when(active, |d| d.child(ic("circle-check", 16., CTA))),
        )
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.select_build(server_id, pick);
            this.build_picker_open = false;
            cx.notify();
        }))
        .into_any_element()
}

fn tag(label: impl Into<SharedString>, color: u32) -> AnyElement {
    div()
        .h(px(16.))
        .px(px(4.))
        .rounded(px(R_SM))
        .bg(rgba((color << 8) | 0x18))
        .border_1()
        .border_color(rgba((color << 8) | 0x44))
        .flex()
        .items_center()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(8.))
        .text_color(rgb(color))
        .child(label.into())
        .into_any_element()
}
