//! Quoting a chat line a moderator pointed at in game.
//!
//! The client points at evidence, it doesn't supply it: the mod sends sender,
//! timestamp and a hash, and the text that lands in the case comes from the
//! agent's own chat capture. A right-click can't fabricate a conversation.
//!
//! If the capture doesn't have the line yet, request one and record the quote
//! anyway — it fills in when the capture arrives.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use chrono::{DateTime, Utc};
use schema::PERM_CASES_CHAT;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

/// How far the client and server clocks may drift and still be the same line.
/// Widen this and the window starts catching the neighbouring message instead.
const MATCH_WINDOW_SECS: i64 = 5;

#[derive(Deserialize)]
pub struct QuoteReq {
    pub sender: String,
    pub at: DateTime<Utc>,
    /// The client's hash of the text. Not evidence — a cross-check that the mod
    /// and the server mean the same line.
    #[serde(default)]
    pub hash: String,
}

pub async fn quote(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<QuoteReq>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CASES_CHAT)?;
    let case = crate::db::get_case(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("case".into()))?;

    let found =
        crate::db::cases::message_near(&state.db, id, &req.sender, req.at, MATCH_WINDOW_SECS)
            .await?;

    // No line yet: ask for a capture. The event gets written either way — the
    // moderator pointing at something is part of the record even if the text
    // only shows up later.
    if found.is_none() {
        if let Ok(Some(target)) = crate::db::get_user(&state.db, case.target_id).await {
            crate::agent_link::cases::request_chat(&state, &case, target.mc_uuid, 600);
        }
    }

    crate::cases::event(
        &state,
        id,
        admin.actor.id(),
        &admin.actor.label(),
        "game",
        "quote",
        json!({
            "sender": req.sender,
            "at": req.at,
            "hash": req.hash,
            "content": found.as_ref().map(|m| m.content.clone()),
            "matched": found.is_some(),
        }),
    )
    .await?;

    Ok(Json(json!({ "ok": true, "matched": found.is_some() })))
}
