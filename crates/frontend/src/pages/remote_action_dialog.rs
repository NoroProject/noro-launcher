//! "An admin is asking you to do something" modal: restart, cache reset, and so on.

use super::common::Cx;
use crate::components::btn;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight};
use schema::RemoteAction;

pub fn dialog(ui: &LauncherUI, cx: &mut Cx) -> Option<AnyElement> {
    let prompt = ui.remote_action_prompt.as_ref()?;

    let (title, desc) = match prompt.action {
        RemoteAction::VerifyIntegrity => (
            "Сверка файлов",
            "Администратор запросил автоматическую проверку целостности файлов.",
        ),
        RemoteAction::ClearAssetCache => (
            "Сброс кэша ассетов",
            "Администратор просит очистить кэш ассетов для исправления возможных сбоев.",
        ),
        RemoteAction::ReinstallBuild => (
            "Переустановка сборки",
            "Администратор просит полностью переустановить текущую сборку.",
        ),
        RemoteAction::RestartLauncher => (
            "Перезапуск лаунчера",
            "Администратор запросил перезапуск лаунчера.",
        ),
        RemoteAction::KillGame => (
            "Завершение процесса игры",
            "Администратор просит принудительно закрыть запущенный процесс Minecraft.",
        ),
    };

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
                    .w(px(460.))
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
                            .child(title),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(TEXT_PRIMARY))
                            .child(format!(
                                "Запрос от администратора: {}",
                                prompt.actor_username
                            )),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(TEXT_MUTED))
                            .child(desc),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(btn(
                                "remote-action-accept",
                                "Выполнить",
                                true,
                                cx.listener(|this, _e, _w, cx| {
                                    this.answer_remote_action(true);
                                    cx.notify();
                                }),
                            ))
                            .child(btn(
                                "remote-action-decline",
                                "Отклонить",
                                false,
                                cx.listener(|this, _e, _w, cx| {
                                    this.answer_remote_action(false);
                                    cx.notify();
                                }),
                            )),
                    ),
            )
            .into_any_element(),
    )
}
