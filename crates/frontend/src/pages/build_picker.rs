//! Build selector in the bottom bar. A dropdown rather than a row of pills
//! because a server can carry a dozen builds, and hidden entirely below two —
//! ordinary players only ever see one.

use super::common::Cx;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight};
use i18n::t;
use schema::{BuildOption, ServerEntry};
use uuid::Uuid;

pub fn build_picker(ui: &LauncherUI, server: &ServerEntry, cx: &mut Cx) -> Option<AnyElement> {
    if server.available_builds.len() < 2 {
        return None;
    }

    let active = ui
        .selected_build
        .get(&server.id)
        .copied()
        .flatten()
        .or(server.current_build_id);
    let current = server
        .available_builds
        .iter()
        .find(|b| Some(b.id) == active);

    Some(
        div()
            .relative()
            .child(trigger(server.id, current, ui.build_picker_open, cx))
            .when(ui.build_picker_open, |d| d.child(menu(server, active, cx)))
            .into_any_element(),
    )
}

/// The menu opens upward — the bar already sits against the bottom of the
/// window.
fn trigger(server_id: Uuid, current: Option<&BuildOption>, open: bool, cx: &mut Cx) -> AnyElement {
    let label = current
        .map(|b| b.version.clone())
        .unwrap_or_else(|| "—".into());
    let preview = current.is_some_and(|b| !b.published);

    div()
        .id("build-picker-trigger")
        .h(px(32.))
        .px(px(12.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .flex()
        .items_center()
        .gap(px(8.))
        .bg(rgb(if open { BG_CARD_HOV } else { BG_INPUT }))
        .border_1()
        .border_color(rgb(if open { CTA } else { BORDER }))
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(9.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(TEXT_MUTED))
                .child(t("game-build")),
        )
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(if preview { ACCENT } else { TEXT_PRIMARY }))
                .child(label),
        )
        .child(ic(
            if open { "chevron-up" } else { "chevron-down" },
            12.,
            TEXT_MUTED,
        ))
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            let _ = server_id;
            this.build_picker_open = !this.build_picker_open;
            cx.notify();
        }))
        .into_any_element()
}

fn menu(server: &ServerEntry, active: Option<Uuid>, cx: &mut Cx) -> AnyElement {
    let server_id = server.id;
    let options: Vec<AnyElement> = server
        .available_builds
        .iter()
        .map(|build| option(server_id, build, active == Some(build.id), cx))
        .collect();

    div()
        .id("build-picker-menu")
        .absolute()
        .bottom(px(40.))
        .left(px(0.))
        .w(px(240.))
        // Capped and scrolled: an active server accumulates enough builds that
        // the list would otherwise run off the top of the window.
        .max_h(px(280.))
        .overflow_y_scroll()
        .p(px(4.))
        .rounded(px(R_SM))
        .bg(rgba(0x0b1626f5))
        .border_1()
        .border_color(rgb(BORDER))
        .flex()
        .flex_col()
        .gap(px(2.))
        .children(options)
        .into_any_element()
}

fn option(server_id: Uuid, build: &BuildOption, active: bool, cx: &mut Cx) -> AnyElement {
    // Unpublished builds carry the accent even when not selected — the player
    // should see they're picking a preview before launching, not after.
    let build_id = build.id;

    div()
        .id(("build-option", build_id.as_u128() as u64))
        .h(px(32.))
        .px(px(10.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .flex()
        .items_center()
        .gap(px(8.))
        .bg(rgb(if active { BG_CARD_HOV } else { BG_INPUT }))
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .child(
            div()
                .flex_1()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(if active { CTA } else { TEXT_PRIMARY }))
                .child(build.version.clone()),
        )
        .when(!build.published, |d| {
            d.child(
                div()
                    .font_family(FONT_PIXEL_ALT)
                    .text_size(px(8.))
                    .text_color(rgb(ACCENT))
                    .child(t("game-build-preview")),
            )
        })
        .when(active, |d| d.child(ic("circle-check", 12., CTA)))
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.select_build(server_id, Some(build_id));
            this.build_picker_open = false;
            cx.notify();
        }))
        .into_any_element()
}
