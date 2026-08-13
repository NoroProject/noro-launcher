//! Выбор версии сборки в нижней панели.
//!
//! Показывается, только когда версий больше одной: у обычного игрока доступна
//! ровно одна, и пустой селектор был бы шумом на самом видном месте экрана.

use super::common::Cx;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent, FontWeight};
use schema::{BuildOption, ServerEntry};
use uuid::Uuid;

/// Ряд версий. Пусто, если выбирать не из чего.
pub fn build_picker(ui: &LauncherUI, server: &ServerEntry, cx: &mut Cx) -> Option<AnyElement> {
    if server.available_builds.len() < 2 {
        return None;
    }

    // Выбранная версия, иначе текущая опубликованная.
    let active = ui
        .selected_build
        .get(&server.id)
        .copied()
        .flatten()
        .or(server.current_build_id);

    let row = server
        .available_builds
        .iter()
        .fold(div().flex().items_center().gap(px(8.)), |row, build| {
            row.child(pill(server.id, build, active == Some(build.id), cx))
        });

    Some(row.into_any_element())
}

fn pill(server_id: Uuid, build: &BuildOption, active: bool, cx: &mut Cx) -> AnyElement {
    // Превью выделяется акцентом даже невыбранным: игрок должен видеть, что
    // берёт неопубликованную версию, до запуска, а не после.
    let accent = if build.published { CTA } else { ACCENT };
    let build_id = build.id;

    div()
        .id(("build-pill", build_id.as_u128() as u64))
        .h(px(32.))
        .px(px(12.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .flex()
        .items_center()
        .gap(px(6.))
        .bg(if active {
            rgb(BG_CARD_HOV)
        } else {
            rgb(BG_INPUT)
        })
        .border_1()
        .border_color(rgb(if active { accent } else { BORDER }))
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(10.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(if active { accent } else { TEXT_SECONDARY }))
                .child(build.version.clone()),
        )
        .when(!build.published, |d| {
            d.child(
                div()
                    .font_family(FONT_PIXEL_ALT)
                    .text_size(px(8.))
                    .text_color(rgb(ACCENT))
                    .child("PREVIEW"),
            )
        })
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.select_build(server_id, Some(build_id));
            cx.notify();
        }))
        .into_any_element()
}
