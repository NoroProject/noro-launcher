//! Модалка «админ просит логи».
//!
//! Кнопка «Посмотреть, что отправится» показывает ровно тот текст, который
//! уйдёт, — уже очищенный. Без неё фича неотличима от слежки; с ней игрок
//! видит `C:\Users\*****` вместо своего имени.

use super::common::Cx;
use crate::components::btn;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight};
use i18n::t;

pub fn dialog(ui: &LauncherUI, cx: &mut Cx) -> Option<AnyElement> {
    let prompt = ui.log_request_prompt.as_ref()?;

    let files = prompt
        .files
        .iter()
        .map(|(name, size)| format!("{name} — {} КБ", size / 1024))
        .collect::<Vec<_>>()
        .join("\n");

    Some(
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba((OVERLAY << 8) | 0xcc))
            .child(
                div()
                    .w(px(520.))
                    .overflow_hidden()
                    .bg(rgb(BG_PANEL))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .rounded(px(12.))
                    .p(px(24.))
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(
                        div()
                            .font_family(FONT_PIXEL_ALT)
                            .text_size(px(15.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(CTA))
                            .child(if prompt.forced {
                                t("logreq-title-forced")
                            } else {
                                t("logreq-title")
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(TEXT_PRIMARY))
                            .child(prompt.actor_username.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(TEXT_MUTED))
                            .child(prompt.reason.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(rgb(TEXT_MUTED))
                            .child(t("logreq-not-collected")),
                    )
                    .child(if ui.log_request_preview_open {
                        preview(&prompt.preview)
                    } else {
                        div()
                            .text_size(px(11.))
                            .font_family(FONT_PIXEL_ALT)
                            .text_color(rgb(TEXT_SECONDARY))
                            .child(files)
                            .into_any_element()
                    })
                    .child(buttons(ui, cx)),
            )
            .into_any_element(),
    )
}

/// Ровно тот текст, который уедет.
fn preview(text: &str) -> AnyElement {
    div()
        .h(px(240.))
        .overflow_hidden()
        .bg(rgb(BG_INPUT))
        .rounded(px(6.))
        .p(px(12.))
        .text_size(px(10.))
        .font_family(FONT_PIXEL_ALT)
        .text_color(rgb(TEXT_SECONDARY))
        .child(text.chars().take(4000).collect::<String>())
        .into_any_element()
}

fn buttons(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let forced = ui.log_request_prompt.as_ref().is_some_and(|p| p.forced);

    div()
        .flex()
        .gap(px(8.))
        .child(btn(
            "logreq-preview",
            t("logreq-preview"),
            false,
            cx.listener(|this, _e, _w, cx| {
                this.log_request_preview_open = !this.log_request_preview_open;
                cx.notify();
            }),
        ))
        // Принудительный сбор уже произошёл — предлагать «отправить» было бы
        // враньём, и остаётся только закрыть окно.
        .when(!forced, |d| {
            d.child(btn(
                "logreq-send",
                t("logreq-send"),
                true,
                cx.listener(|this, _e, _w, cx| {
                    this.answer_log_request(true);
                    cx.notify();
                }),
            ))
            .child(btn(
                "logreq-decline",
                t("logreq-decline"),
                false,
                cx.listener(|this, _e, _w, cx| {
                    this.answer_log_request(false);
                    cx.notify();
                }),
            ))
        })
        .when(forced, |d| {
            d.child(btn(
                "logreq-close",
                t("logreq-close"),
                false,
                cx.listener(|this, _e, _w, cx| {
                    this.dismiss_log_request();
                    cx.notify();
                }),
            ))
        })
        .into_any_element()
}
