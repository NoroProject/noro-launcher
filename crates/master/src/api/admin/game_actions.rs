//! Действия из админки в игру: kick, tell, announce.

use crate::api::auth::AdminAuth;
use crate::audit;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use schema::{PERM_GAME_ANNOUNCE, PERM_GAME_KICK, PERM_GAME_TELL};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct KickReq {
    pub server_id: Option<Uuid>,
    pub target: Uuid,
    pub message: String,
}

#[derive(Deserialize)]
pub struct TellReq {
    pub server_id: Option<Uuid>,
    pub target: Uuid,
    pub message: String,
}

#[derive(Deserialize)]
pub struct AnnounceReq {
    pub server_id: Option<Uuid>,
    pub message: String,
}

/// POST /api/admin/game/kick
pub async fn kick(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<KickReq>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_GAME_KICK)?;
    if req.message.trim().is_empty() {
        return Err(AppError::BadRequest("message cannot be empty".into()));
    }
    let target_server = req.server_id.or_else(|| state.roster.locate(req.target));
    crate::agent_link::notify::kick(&state, target_server, req.target, req.message.clone());
    audit::record(
        &state,
        &admin.actor,
        audit::actions::GAME_KICK,
        None,
        json!({ "target": req.target, "message": req.message, "server_id": target_server }),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}

/// POST /api/admin/game/tell
pub async fn tell(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<TellReq>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_GAME_TELL)?;
    if req.message.trim().is_empty() {
        return Err(AppError::BadRequest("message cannot be empty".into()));
    }
    let target_server = req.server_id.or_else(|| state.roster.locate(req.target));
    crate::agent_link::notify::tell(&state, target_server, req.target, req.message.clone());
    audit::record(
        &state,
        &admin.actor,
        audit::actions::GAME_TELL,
        None,
        json!({ "target": req.target, "message": req.message, "server_id": target_server }),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}

/// POST /api/admin/game/announce
pub async fn announce(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<AnnounceReq>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_GAME_ANNOUNCE)?;
    if req.message.trim().is_empty() {
        return Err(AppError::BadRequest("message cannot be empty".into()));
    }
    crate::agent_link::notify::announce(&state, req.server_id, req.message.clone());
    audit::record(
        &state,
        &admin.actor,
        audit::actions::GAME_ANNOUNCE,
        None,
        json!({ "message": req.message, "server_id": req.server_id }),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}
