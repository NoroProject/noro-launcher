//! Re-confirming identity before entering someone else's account.
//!
//! A passkey is the main path; a one-time recovery code covers instances where
//! WebAuthn isn't available at all (plain `http://`, no domain).
//!
//! Confirming opens a 15-minute window rather than authorising a single entry.
//! On the recovery-code path that's the difference between ten impersonations
//! and ten burnt codes, on exactly the instances that have no other second
//! factor to fall back on.

use crate::api::auth::passkeys::LoginVerifyReq;
use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

/// Whether the window is open, and what could open it.
pub async fn status(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "active": crate::db::step_up_active(&state.db, user.user_id).await?,
        "passkey_available": state.webauthn.is_some(),
        "recovery_codes_left": crate::db::recovery_codes_left(&state.db, user.user_id).await?,
        "window_minutes": crate::db::STEP_UP_WINDOW_MINS,
    })))
}

pub async fn confirm_passkey(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<LoginVerifyReq>,
) -> AppResult<Json<Value>> {
    let owner = crate::api::auth::passkeys::verify_login(&state, &req).await?;
    // `verify_login` only proves the key is valid, not that it's this user's.
    if owner != user.user_id {
        return Err(AppError::Forbidden(
            "this key belongs to another account".into(),
        ));
    }
    crate::db::open_step_up(&state.db, user.user_id, "passkey").await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct CodeReq {
    pub code: String,
}

/// The code is consumed whether or not it turns out to belong to this user.
pub async fn confirm_recovery(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CodeReq>,
) -> AppResult<Json<Value>> {
    let owner =
        crate::db::consume_recovery_code(&state.db, &user.profile.username, req.code.trim())
            .await?;
    if owner != Some(user.user_id) {
        return Err(AppError::Unauthorized("that code did not match".into()));
    }
    crate::db::open_step_up(&state.db, user.user_id, "recovery_code").await?;

    let left = crate::db::recovery_codes_left(&state.db, user.user_id).await?;
    Ok(Json(json!({ "ok": true, "codes_left": left })))
}
