//! Диалог подтверждения входа в чужой аккаунт и баннер о том, что вход активен.
//!
//! Диалог здесь работает вторым фактором: он закрывает случай «злоумышленник
//! получил веб-сессию админа, но не доступ к его машине». Поэтому подтверждение
//! спрашивается именно тут, а не в браузере.

use super::common::Cx;
use crate::components::btn;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight};
use i18n::t;

/// Модальный запрос. `None`, если ничего не ждём.
pub fn dialog(ui: &LauncherUI, cx: &mut Cx) -> Option<AnyElement> {
    let prompt = ui.impersonate_prompt.as_ref()?;

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
                    .w(px(420.))
                    .bg(rgb(BG_PANEL))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .rounded(px(12.))
                    .p(px(24.))
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(ic("user", 18., CTA))
                            .child(
                                div()
                                    .font_family(FONT_PIXEL_ALT)
                                    .text_size(px(16.))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(CTA))
                                    .child(t("impersonate-title")),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(rgb(TEXT_PRIMARY))
                            .child(prompt.target_username.clone()),
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
                            .child(format!("{} c", prompt.expires_in_secs)),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(btn(
                                "impersonate-accept",
                                t("impersonate-accept"),
                                true,
                                cx.listener(|this, _e, _w, cx| {
                                    this.answer_impersonate(true);
                                    cx.notify();
                                }),
                            ))
                            .child(btn(
                                "impersonate-decline",
                                t("impersonate-decline"),
                                false,
                                cx.listener(|this, _e, _w, cx| {
                                    this.answer_impersonate(false);
                                    cx.notify();
                                }),
                            )),
                    ),
            )
            .into_any_element(),
    )
}

// Отметка «вы в чужом аккаунте» — в рамке окна: она относится к окну целиком,
// а не к странице. См. `components::window_chrome`.
