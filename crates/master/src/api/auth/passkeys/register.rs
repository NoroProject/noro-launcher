//! Привязка ключа в кабинете и управление списком.

use super::{reject, ChallengeRes};
use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use base64::Engine;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use webauthn_rs::prelude::*;

/// Опции для `navigator.credentials.create()`.
pub async fn register_options(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ChallengeRes<CreationChallengeResponse>>> {
    crate::api::auth::oauth::methods::require_passkey_enabled(&state).await?;
    let profile = crate::db::load_profile(&state.db, user.user_id).await?;

    // Уже привязанные ключи исключаются: иначе игрок молча заводит второй ключ
    // на том же устройстве и путается, какой из них удалять.
    let existing = crate::db::passkeys_for_user(&state.db, user.user_id).await?;
    let exclude: Vec<CredentialID> = existing
        .iter()
        .filter_map(|p| serde_json::from_value::<Passkey>(p.credential.clone()).ok())
        .map(|p| p.cred_id().clone())
        .collect();

    let (options, reg_state) = state
        .webauthn()?
        .start_passkey_registration(
            profile.id,
            &profile.username,
            &profile.username,
            Some(exclude),
        )
        .map_err(reject)?;

    let state_id = crate::db::save_webauthn_state(
        &state.db,
        "register",
        Some(user.user_id),
        &serde_json::to_value(&reg_state)?,
    )
    .await?;

    Ok(Json(ChallengeRes { state_id, options }))
}

#[derive(Deserialize)]
pub struct RegisterVerifyReq {
    pub state_id: Uuid,
    pub name: String,
    pub credential: RegisterPublicKeyCredential,
}

/// Проверить ответ аутентификатора и сохранить ключ.
pub async fn register_verify(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<RegisterVerifyReq>,
) -> AppResult<Json<Value>> {
    let (owner, raw_state) = crate::db::take_webauthn_state(&state.db, req.state_id, "register")
        .await?
        .ok_or_else(|| AppError::BadRequest("The challenge has expired".into()))?;

    if owner != Some(user.user_id) {
        return Err(AppError::Forbidden(
            "Invalid user for this challenge".into(),
        ));
    }

    let reg_state: PasskeyRegistration = serde_json::from_value(raw_state)?;
    let passkey = state
        .webauthn()?
        .finish_passkey_registration(&req.credential, &reg_state)
        .map_err(reject)?;

    let name = match req.name.trim() {
        "" => "Passkey".to_string(),
        n => n.chars().take(64).collect(),
    };
    let saved = crate::db::create_passkey(
        &state.db,
        user.user_id,
        &name,
        &encode_cred_id(passkey.cred_id()),
        &serde_json::to_value(&passkey)?,
    )
    .await?;

    Ok(Json(json!(saved)))
}

/// Ключи пользователя — без ключевого материала.
pub async fn list_passkeys(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let passkeys = crate::db::list_passkeys_for_user(&state.db, user.user_id).await?;
    Ok(Json(json!(passkeys)))
}

pub async fn delete_passkey(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let ok = crate::db::delete_passkey(&state.db, id, user.user_id).await?;
    Ok(Json(json!({ "success": ok })))
}

/// base64url без выравнивания — тот же вид, в котором id приходит от браузера.
pub fn encode_cred_id(id: &CredentialID) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(id.as_ref())
}
