//! Глобальные настройки лаунчера (без вкладок сервера).

use super::common::{panel, Cx};
use super::settings_panel::settings_panel;
use super::settings_rows::row;
use crate::components::{btn, version_badge};
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, FontWeight};
use i18n::t;

pub fn page(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .size_full()
        .bg(rgb(CONTENT_FALLBACK))
        .flex()
        .flex_col()
        .child(header())
        .child(
            div()
                .flex_1()
                .min_h_0()
                .px(px(32.))
                .py(px(20.))
                .flex()
                .flex_col()
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(16.))
                        .child(settings_panel(ui, cx))
                        .when_some(ui.update_available.clone(), |d, v| {
                            d.child(update_panel(v.version, cx))
                        })
                        .child(version_badge()),
                ),
        )
        .into_any_element()
}

fn header() -> AnyElement {
    div()
        .h(px(72.))
        .px(px(32.))
        .flex()
        .items_center()
        .gap(px(12.))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(ic("settings", 18., TEXT_MUTED))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(18.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(t("settings-title")),
        )
        .into_any_element()
}

fn update_panel(version: String, cx: &mut Cx) -> AnyElement {
    panel()
        .child(row(
            "download",
            t("settings-update"),
            t("settings-update-hint"),
            btn(
                "do-update",
                t("settings-install-update"),
                true,
                cx.listener(|this, _e, _w, cx| {
                    this.install_update();
                    cx.notify();
                }),
            )
            .into_any_element(),
            false,
        ))
        .child(
            div().px(px(20.)).pb(px(16.)).child(
                div()
                    .font_family(FONT_PIXEL_ALT)
                    .text_size(px(11.))
                    .text_color(rgb(TEXT_MUTED))
                    .child(format!("v{}", version.trim_start_matches("launcher-v"))),
            ),
        )
        .into_any_element()
}
