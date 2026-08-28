use super::common::{progress_label, Cx};
use crate::components::{cta_button, mascot, progress_bar, stage_row, Mood};
use crate::state::SyncUiState;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight};
use i18n::t;
use uuid::Uuid;

pub fn sync_overlay(server_id: Uuid, sync: &SyncUiState, cx: &mut Cx) -> AnyElement {
    div()
        .absolute()
        .left(px(32.))
        .right(px(32.))
        .bottom(px(120.))
        .rounded(px(R_SM))
        .bg(rgba(0x0b1626f8)) // More opaque
        .border_1()
        .border_color(rgb(if sync.failed.is_some() { ERROR } else { BORDER }))
        .p(px(20.))
        .flex()
        .items_start()
        .gap(px(16.))
        // Маскот рядом с полосой: пока идёт долгая закачка, он показывает, что
        // лаунчер занят делом, а не завис. При ошибке он не к месту.
        .when(sync.failed.is_none(), |d| {
            d.child(mascot(Mood::Loading, 56.))
        })
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .font_family(FONT_PIXEL_ALT)
                                .text_size(px(14.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(if sync.failed.is_some() {
                                    ERROR
                                } else {
                                    TEXT_PRIMARY
                                }))
                                .child(if sync.failed.is_some() {
                                    t("sync-failed")
                                } else if sync.stage.is_empty() {
                                    t("game-preparing")
                                } else {
                                    sync.stage.to_uppercase()
                                }),
                        )
                        .child(div().flex_1())
                        .child(
                            div()
                                .font_family(FONT_PIXEL_ALT)
                                .text_size(px(12.))
                                .text_color(rgb(TEXT_MUTED))
                                .child(progress_label(sync)),
                        ),
                )
                .child(progress_bar(sync.fraction()))
                // Полосы стадий под общей: видно, что качается прямо сейчас, и
                // что осталось. Пока стадий нет (идёт проверка файлов) — пусто.
                .when(!sync.stages.is_empty() && sync.failed.is_none(), |d| {
                    d.child(
                        div().flex().flex_col().gap(px(4.)).children(
                            sync.stages
                                .iter()
                                .map(|(stage, (done, total))| stage_row(*stage, *done, *total)),
                        ),
                    )
                })
                .when(!sync.detail.is_empty(), |d| {
                    d.child(
                        div()
                            .text_xs()
                            .truncate()
                            .font_family(FONT_PIXEL_ALT)
                            .text_color(rgb(TEXT_MUTED))
                            .child(sync.detail.clone()),
                    )
                })
                .when_some(sync.failed.clone(), |d, e| {
                    d.child(
                        div()
                            .mt(px(8.))
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(px(16.))
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .font_family(FONT_PIXEL_ALT)
                                    .text_color(rgb(ERROR))
                                    .child(format!("Error: {e}")),
                            )
                            .child(cta_button(
                                "sync-retry-btn",
                                Some("rotate-ccw"),
                                t("retry"),
                                cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                                    this.launch(server_id);
                                    cx.notify();
                                }),
                            )),
                    )
                }),
        )
        .into_any_element()
}
