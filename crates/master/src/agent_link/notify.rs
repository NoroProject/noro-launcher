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

async fn load_user(state: &AppState, row: &PunishmentRow) -> Option<UserRow> {
    // Пустой хаб — обычное дело: серверы могут быть выключены. Тогда и в базу
    // ходить незачем.
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
