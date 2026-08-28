//! What the master does with frames from an agent.
//!
//! No handler here replies to the agent. The agent decides on its own; the
//! master only finds out what happened.

use super::proto::FromAgent;
use crate::db::GameServerRow;
use crate::state::AppState;

/// An unrecognised frame is not an error — the agent may be ahead of us on
/// version, and tearing down the channel over it loses the frames we do
/// understand.
pub async fn handle(state: &AppState, server: &GameServerRow, frame: &str) {
    match serde_json::from_str::<FromAgent>(frame) {
        Ok(msg) => apply(state, server, msg).await,
        Err(e) => tracing::debug!(server = %server.name, error = %e, "skipped agent frame"),
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
                tracing::warn!(server = %server.name, player = %uuid, error = %e, "failed to open player session");
            }
            tracing::debug!(
                server = %server.name,
                player = %uuid,
                vanished,
                ip = ip_hash.as_deref().unwrap_or("-"),
                "player joined"
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
                tracing::warn!(server = %server.name, player = %uuid, error = %e, "failed to close player session");
            }
            tracing::debug!(
                server = %server.name,
                player = %uuid,
                reason = reason.as_deref().unwrap_or("-"),
                "player left"
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
        // Anything shorter is only logged: a server that skips a few ticks
        // recovers on its own, and restarting it would cost more than the stall.
        FromAgent::TickStall { stalled_secs } => {
            tracing::warn!(
                server = %server.name,
                stalled_secs,
                "game thread stalled"
            );
            if stalled_secs >= 60 {
                if let Err(e) = crate::wrapper::ops::power(state, server.server_id, "restart").await
                {
                    tracing::error!(server = %server.name, error = %e, "failed to restart stalled server");
                }
            }
        }
    }
}

/// Put a moderator back into review mode when they log in.
///
/// The case lock lives in the database, but the agent's review session only
/// lives in server memory. A server restart, or claiming the case from the
/// site, separates the two: the panel says "in progress" while `/case …`
/// answers that you aren't reviewing anything.
async fn restore_case_mode(state: &AppState, server: &GameServerRow, mc_uuid: uuid::Uuid) {
    let Ok(Some(user)) = crate::db::user_by_mc_uuid(&state.db, mc_uuid).await else {
        return;
    };
    let cases = match crate::db::cases::claimed_on_server(&state.db, user.id, server.id).await {
        Ok(cases) => cases,
        Err(e) => {
            tracing::warn!(player = %mc_uuid, error = %e, "failed to read moderator's cases");
            return;
        }
    };
    for case in &cases {
        tracing::debug!(player = %mc_uuid, case = %case.id, "restoring review mode");
        super::cases::assigned(state, case, mc_uuid).await;
    }
}
