//! Выбор модели скина: классическая (Стив) или тонкая (Алекс).
//!
//! Отдельным файлом, а не строчкой в `profile_skin`: там уже карточка превью,
//! пресеты, перетаскивание и переименование — ещё один блок сделал бы её
//! нечитаемой.
//!
//! Переключение не трогает картинку: меняется только ширина рук. Заставлять
//! игрока перезаливать файл ради этого нельзя — скина у него на диске может и
//! не быть, если тот приехал по нику.

use super::common::Cx;
use crate::components::btn;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement};
use i18n::t;

/// Ряд из двух кнопок. Пусто, если скина нет: переключать нечего, а показывать
/// выбор поверх общего Стива значит обещать то, чего игрок не загружал.
pub fn model_row(ui: &LauncherUI, cx: &mut Cx) -> Option<AnyElement> {
    ui.user.as_ref()?.skin_url.as_ref()?;
    let slim = ui.user.as_ref().is_some_and(|u| u.skin_slim);

    Some(
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(
                div()
                    .font_family(FONT_PIXEL_ALT)
                    .text_size(px(12.))
                    .text_color(rgb(TEXT_MUTED))
                    .child(t("profile-skin-model")),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(choice(ui, cx, false, !slim, "profile-skin-model-classic"))
                    .child(choice(ui, cx, true, slim, "profile-skin-model-slim")),
            )
            .into_any_element(),
    )
}

fn choice(
    ui: &LauncherUI,
    cx: &mut Cx,
    slim: bool,
    active: bool,
    label_key: &'static str,
) -> AnyElement {
    // Выбранную кнопку не гасим: повторное нажатие безвредно, а серая кнопка
    // рядом с активной читается как «недоступно», а не как «уже выбрано».
    let busy = ui.skin_uploading;
    div()
        .flex_1()
        .child(btn(
            if slim {
                "skin-model-slim"
            } else {
                "skin-model-classic"
            },
            t(label_key),
            active,
            cx.listener(move |this, _, _, cx| {
                if !busy {
                    this.set_skin_model(slim, cx);
                }
            }),
        ))
        .into_any_element()
}
