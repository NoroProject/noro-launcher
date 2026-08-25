//! Что мастер делает с кадрами, пришедшими от агента.
//!
//! Отдельно от `session.rs`, чтобы та осталась про сокет: разбор кадра и
//! реакция на него растут вместе с планом, а работа с соединением — нет.
//!
//! Ни один обработчик не отвечает агенту: канал односторонний по смыслу,
//! решения агент принимает сам, а мастер лишь узнаёт о случившемся.

use super::proto::FromAgent;
use crate::db::GameServerRow;
use crate::state::AppState;

/// Разобрать текстовый кадр и применить его.
///
/// Незнакомый кадр — не ошибка: агент может уехать вперёд по версии, и рвать
/// из-за этого канал значит терять и те кадры, которые мы понимаем.
pub async fn handle(state: &AppState, server: &GameServerRow, frame: &str) {
    match serde_json::from_str::<FromAgent>(frame) {
        Ok(msg) => apply(state, server, msg).await,
        Err(e) => tracing::debug!(server = %server.name, error = %e, "кадр от агента пропущен"),
    }
}

async fn apply(state: &AppState, server: &GameServerRow, msg: FromAgent) {
    match msg {
        FromAgent::PlayerJoin {
            uuid,
            ip_hash,
            vanished,
        } => {
            state.roster.join(server.id, uuid, vanished);
            if let Err(e) =
                crate::db::game_sessions::start_session(&state.db, uuid, server.id).await
            {
                tracing::warn!(server = %server.name, player = %uuid, error = %e, "не удалось открыть сессию игрока");
            }
            tracing::debug!(
                server = %server.name,
                player = %uuid,
                vanished,
                ip = ip_hash.as_deref().unwrap_or("-"),
                "игрок вошёл"
            );
            restore_case_mode(state, server, uuid).await;
        }
        FromAgent::PlayerLeave { uuid, reason } => {
            state.roster.leave(server.id, uuid);
            if let Err(e) = crate::db::game_sessions::end_session(
                &state.db,
                uuid,
                server.id,
                reason.as_deref().unwrap_or("leave"),
            )
            .await
            {
                tracing::warn!(server = %server.name, player = %uuid, error = %e, "не удалось закрыть сессию игрока");
            }
            tracing::debug!(
                server = %server.name,
                player = %uuid,
                reason = reason.as_deref().unwrap_or("-"),
                "игрок вышел"
            );
        }
        FromAgent::CaseClaim { case, moderator } => {
            super::cases::claim_from_game(state, case, moderator).await;
        }
        FromAgent::CaseAction {
            case,
            moderator,
            kind,
            payload,
        } => super::cases::action(state, case, moderator, &kind, payload).await,
        FromAgent::CaseChatSlice { case, messages } => {
            super::cases::chat_slice(state, case, &messages).await
        }
        FromAgent::CaseInventory {
            case,
            moderator,
            items,
        } => super::cases::inventory(state, server, case, moderator, items).await,
        // Рестарт по зависанию — п.39, он приходит вместе с расписаниями и
        // правом дёргать враппер. Пока это сигнал в журнал: сервер, который не
        // тикает минуту, обязан быть виден оператору, даже если чинить его
        // некому.
        FromAgent::TickStall { stalled_secs } => {
            tracing::warn!(
                server = %server.name,
                stalled_secs,
                "игровой поток не двигался — перезапускаем зависший сервер через wrapper"
            );
            if stalled_secs >= 60 {
                if let Err(e) = crate::wrapper::ops::power(state, server.server_id, "restart").await
                {
                    tracing::error!(server = %server.name, error = %e, "не удалось перезапустить зависший сервер");
                }
            }
        }
    }
}

/// Вернуть модератору режим разбора при входе в игру.
///
/// Замок на деле ставится на мастере и живёт в базе, а сессия разбора у
/// агента — в памяти сервера. Их разводит любой перезапуск сервера и любое
/// взятие дела с сайта: панель показывает «в работе», а команды `/case …`
/// отвечают «вы не ведёте разбор». Кадр на входе снова их сводит.
async fn restore_case_mode(state: &AppState, server: &GameServerRow, mc_uuid: uuid::Uuid) {
    let Ok(Some(user)) = crate::db::user_by_mc_uuid(&state.db, mc_uuid).await else {
        return;
    };
    let cases = match crate::db::cases::claimed_on_server(&state.db, user.id, server.id).await {
        Ok(cases) => cases,
        Err(e) => {
            tracing::warn!(player = %mc_uuid, error = %e, "дела модератора не прочитались");
            return;
        }
    };
    for case in &cases {
        tracing::debug!(player = %mc_uuid, case = %case.id, "возвращаем режим разбора");
        super::cases::assigned(state, case, mc_uuid).await;
    }
}
