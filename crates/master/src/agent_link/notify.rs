//! Pushing punishments out to game servers.
//!
//! One entry point for everyone who punishes — admin panel, account page and
//! the agent itself — so a ban behaves the same however it was issued.

use super::proto::{LivePunishment, ToAgent};
use crate::db::models::UserRow;
use crate::db::punishments::PunishmentRow;
use crate::state::AppState;

/// Errors are only logged: the punishment is already in the database, and
/// failing here because a frame didn't go out would lose it.
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

/// Goes to the same audience as the original: a server where the player is
/// sitting muted has to hear about the lift even if it didn't issue the mute.
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

/// A player's roles, permissions or prefix changed.
///
/// Without this a role granted on the site means nothing in game until the
/// player relogs: `ProfileCache` is filled at login and only refreshes mutes.
///
/// `None` means re-read everyone, which is how an edit to the role itself
/// travels — listing its holders here would repeat a query the agent runs
/// anyway against its own online list.
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
            tracing::warn!(error = %e, "punishment not sent to agents: can't read the player");
            None
        }
    }
}
