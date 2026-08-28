//! Cases over the agent channel: review mode goes out, actions come back.
//!
//! The master is the only side that takes the lock and writes the feed. An
//! in-game `/case` and a button on the site can equally lose the race.

use super::proto::ToAgent;
use crate::db::cases::{CaseRow, IncomingMessage};
use crate::db::GameServerRow;
use crate::state::AppState;
use serde_json::json;
use uuid::Uuid;

/// Case claimed — hand the moderator review mode.
pub async fn assigned(state: &AppState, case: &CaseRow, moderator_mc: Uuid) {
    let Some(game_server_id) = case.game_server_id else {
        return;
    };
    let Ok(Some(target)) = crate::db::get_user(&state.db, case.target_id).await else {
        return;
    };
    let reports = crate::db::case_reports(&state.db, case.id)
        .await
        .unwrap_or_default();
    let last = reports.last();
    let reporter = match last {
        Some(r) => crate::db::get_user(&state.db, r.reporter_id)
            .await
            .ok()
            .flatten(),
        None => None,
    };

    let msg = ToAgent::CaseAssigned {
        case: case.id,
        target: target.mc_uuid,
        target_name: target.mc_username,
        moderator: moderator_mc,
        reason: last.map(|r| r.reason.clone()).unwrap_or_default(),
        world: last.and_then(|r| r.world.clone()),
        x: last.and_then(|r| r.x),
        y: last.and_then(|r| r.y),
        z: last.and_then(|r| r.z),
        reporter: reporter.as_ref().map(|u| u.mc_uuid),
        reporter_name: reporter.map(|u| u.mc_username),
    };
    state.agents.send_to_game_server(&msg, game_server_id);
}

/// Case released or closed — take the moderator out of review mode.
pub fn finished(state: &AppState, case: &CaseRow, moderator_mc: Uuid, closed: bool) {
    let Some(game_server_id) = case.game_server_id else {
        return;
    };
    let msg = ToAgent::CaseFinished {
        case: case.id,
        moderator: moderator_mc,
        closed,
    };
    state.agents.send_to_game_server(&msg, game_server_id);
}

pub fn request_chat(state: &AppState, case: &CaseRow, target_mc: Uuid, before_secs: u32) {
    let Some(game_server_id) = case.game_server_id else {
        return;
    };
    let msg = ToAgent::CaseChatRequest {
        case: case.id,
        target: target_mc,
        before_secs,
    };
    state.agents.send_to_game_server(&msg, game_server_id);
}

pub fn request_inventory(state: &AppState, case: &CaseRow, target_mc: Uuid) {
    let Some(game_server_id) = case.game_server_id else {
        return;
    };
    let msg = ToAgent::CaseInventoryRequest {
        case: case.id,
        target: target_mc,
    };
    state.agents.send_to_game_server(&msg, game_server_id);
}

/// Moderator claimed a case with an in-game command.
pub async fn claim_from_game(state: &AppState, case_id: Uuid, moderator_mc: Uuid) {
    let Ok(Some(moderator)) = crate::db::user_by_mc_uuid(&state.db, moderator_mc).await else {
        return;
    };
    let Ok(Some(case)) = crate::db::get_case(&state.db, case_id).await else {
        return;
    };
    let taken = crate::db::claim_case(&state.db, case_id, moderator.id)
        .await
        .unwrap_or(false);
    if !taken {
        return;
    }
    let _ = crate::cases::event(
        state,
        case_id,
        Some(moderator.id),
        &moderator.mc_username,
        "game",
        "claimed",
        json!({}),
    )
    .await;
    assigned(state, &case, moderator_mc).await;
}

/// An action taken in review mode: teleport, freeze, spectate, inspect.
pub async fn action(
    state: &AppState,
    case_id: Uuid,
    moderator_mc: Uuid,
    kind: &str,
    payload: serde_json::Value,
) {
    let moderator = crate::db::user_by_mc_uuid(&state.db, moderator_mc)
        .await
        .ok()
        .flatten();
    let label = moderator
        .as_ref()
        .map(|u| u.mc_username.clone())
        .unwrap_or_else(|| "—".into());
    let _ = crate::cases::event(
        state,
        case_id,
        moderator.map(|u| u.id),
        &label,
        "game",
        kind,
        payload,
    )
    .await;
}

pub async fn chat_slice(state: &AppState, case_id: Uuid, messages: &[IncomingMessage]) {
    match crate::db::save_messages(&state.db, case_id, messages).await {
        Ok(0) => {}
        Ok(saved) => {
            let _ = crate::cases::event(
                state,
                case_id,
                None,
                "",
                "system",
                "chat_slice",
                json!({ "messages": saved }),
            )
            .await;
        }
        Err(e) => tracing::warn!(case = %case_id, error = %e, "chat slice not saved"),
    }
}

/// Inventory snapshot. Stored in the feed — one JSON blob doesn't need its own
/// table.
pub async fn inventory(
    state: &AppState,
    server: &GameServerRow,
    case_id: Uuid,
    moderator_mc: Option<Uuid>,
    items: serde_json::Value,
) {
    let moderator = match moderator_mc {
        Some(mc) => crate::db::user_by_mc_uuid(&state.db, mc)
            .await
            .ok()
            .flatten(),
        None => None,
    };
    let label = moderator
        .as_ref()
        .map(|u| u.mc_username.clone())
        .unwrap_or_else(|| server.name.clone());
    let _ = crate::cases::event(
        state,
        case_id,
        moderator.map(|u| u.id),
        &label,
        "game",
        "inventory_snapshot",
        items,
    )
    .await;
}
