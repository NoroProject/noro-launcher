//! Повторное подтверждение личности перед входом в чужой аккаунт.
//!
//! Passkey не старше пяти минут — основной путь. Там, где passkey недоступен
//! (инстанс на `http://` без домена), подходит одноразовый recovery-код.
//!
//! Успешное подтверждение открывает окно на 15 минут: иначе десять кодов
//! сгорели бы за десять входов ровно там, где других кодов взять негде.

use crate::api::auth::passkeys::LoginVerifyReq;
use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

/// Открыто ли окно и чем его можно открыть.
pub async fn status(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    Ok(Json(json!({
        "active": crate::db::step_up_active(&state.db, user.user_id).await?,
        "passkey_available": state.webauthn.is_some(),
        "recovery_codes_left": crate::db::recovery_codes_left(&state.db, user.user_id).await?,
        "window_minutes": crate::db::STEP_UP_WINDOW_MINS,
    })))
}

/// Подтвердить passkey.
pub async fn confirm_passkey(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<LoginVerifyReq>,
) -> AppResult<Json<Value>> {
    let owner = crate::api::auth::passkeys::verify_login(&state, &req).await?;
    // Ключ обязан быть свой: чужой подтверждает чужую личность.
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

/// Подтвердить recovery-кодом. Код сгорает — он одноразовый.
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
