//! Рассылка наказаний на игровые серверы.
//!
//! Одна точка для всех, кто наказывает: админка, кабинет и сам агент. Иначе
//! бан из панели долетал бы до игры мгновенно, а бан командой из игры — только
//! к следующему входу, и объяснить эту разницу было бы нечем.

use super::proto::{LivePunishment, ToAgent};
use crate::db::models::UserRow;
use crate::db::punishments::PunishmentRow;
use crate::state::AppState;

/// Наказание выдано. Ошибки только логируем: наказание уже в базе, и падать
/// из-за того, что кадр не ушёл, значит терять сам факт.
pub async fn punished(state: &AppState, row: &PunishmentRow) {
    let Some(user) = load_user(state, row).await else {
        return;
    };
    let msg = ToAgent::Punished {
        punishment: LivePunishment {
            id: row.id,
            target: user.mc_uuid,
            target_name: user.mc_username,
            kind: row.kind.clone(),
            reason: row.reason.clone(),
            actor_label: row.actor_label.clone(),
            created_at: row.created_at,
            expires_at: row.expires_at,
            rule_code: row.rule_code.clone(),
        },
    };
    state.agents.send(&msg, row.server_id);
}

/// Наказание снято. Кадр уходит той же аудитории, что и выдача: сервер, где
/// игрок сидит замученным, обязан узнать о снятии, даже если сам его не выдавал.
pub async fn revoked(state: &AppState, row: &PunishmentRow, actor_label: &str) {
    let Some(user) = load_user(state, row).await else {
        return;
    };
    let msg = ToAgent::Revoked {
        id: row.id,
        target: user.mc_uuid,
        target_name: user.mc_username,
        kind: row.kind.clone(),
        actor_label: actor_label.to_string(),
    };
    state.agents.send(&msg, row.server_id);
}

/// Шаблоны сообщений поменяли — агенты перечитают их сами.
pub fn messages_changed(state: &AppState) {
    state.agents.broadcast(&ToAgent::MessagesChanged);
}

pub fn filters_changed(state: &AppState) {
    state.agents.broadcast(&ToAgent::FiltersChanged);
}

pub fn restart_notice(
    state: &AppState,
    server_id: Option<uuid::Uuid>,
    game_server_id: Option<uuid::Uuid>,
    seconds: u32,
    reason: Option<String>,
) {
    let msg = ToAgent::RestartNotice { seconds, reason };
    if let Some(gs_id) = game_server_id {
        state.agents.send_to_game_server(&msg, gs_id);
    } else if let Some(sid) = server_id {
        state.agents.send(&msg, Some(sid));
    } else {
        state.agents.broadcast(&msg);
    }
}

/// Профиль игрока изменился: роли, права, префикс.
///
/// Без этого выданная на сайте роль не значила в игре ничего до перезахода:
/// `ProfileCache` наполняется на логине и обновляет только муты.
///
/// `None` — перечитать всех. Так уходит правка самой роли: носителей у неё
/// сколько угодно, и перечислять их здесь значит повторить выборку, которую
/// агент всё равно сделает по своему списку онлайна.
pub fn profile_changed(state: &AppState, mc_uuid: Option<uuid::Uuid>) {
    state
        .agents
        .broadcast(&ToAgent::ProfileChanged { uuid: mc_uuid });
}

pub fn kick(state: &AppState, server_id: Option<uuid::Uuid>, target: uuid::Uuid, message: String) {
    let msg = ToAgent::Kick { target, message };
    if let Some(sid) = server_id {
        state.agents.send(&msg, Some(sid));
    } else {
        state.agents.broadcast(&msg);
    }
}

pub fn tell(state: &AppState, server_id: Option<uuid::Uuid>, target: uuid::Uuid, message: String) {
    let msg = ToAgent::Tell { target, message };
    if let Some(sid) = server_id {
        state.agents.send(&msg, Some(sid));
    } else {
        state.agents.broadcast(&msg);
    }
}

pub fn announce(state: &AppState, server_id: Option<uuid::Uuid>, message: String) {
    let msg = ToAgent::Announce { message };
    if let Some(sid) = server_id {
        state.agents.send(&msg, Some(sid));
    } else {
        state.agents.broadcast(&msg);
    }
}

pub fn maintenance_start(
    state: &AppState,
    server_id: Option<uuid::Uuid>,
    game_server_id: Option<uuid::Uuid>,
    countdown_seconds: u32,
    reason: Option<String>,
) {
    let msg = ToAgent::MaintenanceStart {
        countdown_seconds,
        reason,
    };
    if let Some(gs_id) = game_server_id {
        state.agents.send_to_game_server(&msg, gs_id);
    } else if let Some(sid) = server_id {
        state.agents.send(&msg, Some(sid));
    } else {
        state.agents.broadcast(&msg);
    }
}

pub fn maintenance_cancel(
    state: &AppState,
    server_id: Option<uuid::Uuid>,
    game_server_id: Option<uuid::Uuid>,
) {
    let msg = ToAgent::MaintenanceCancel;
    if let Some(gs_id) = game_server_id {
        state.agents.send_to_game_server(&msg, gs_id);
    } else if let Some(sid) = server_id {
        state.agents.send(&msg, Some(sid));
    } else {
        state.agents.broadcast(&msg);
    }
}

async fn load_user(state: &AppState, row: &PunishmentRow) -> Option<UserRow> {
    if state.agents.connected_count() == 0 {
        return None;
    }
    match crate::db::get_user(&state.db, row.user_id).await {
        Ok(user) => user,
        Err(e) => {
            tracing::warn!(error = %e, "наказание не ушло агентам: игрок не читается");
            None
        }
    }
}
