use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

fn random_challenge() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    hex::encode(bytes)
}

#[derive(Serialize)]
pub struct ChallengeOptionsRes {
    pub challenge: String,
    pub rp: Value,
    pub user: Option<Value>,
}

/// Генерация опций для регистрации Passkey в кабинете
pub async fn register_options(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let challenge = random_challenge();
    crate::db::save_challenge(&state.db, &challenge, Some(user.user_id)).await?;

    let u = crate::db::load_profile(&state.db, user.user_id).await?;
    let domain = state
        .config
        .public_url
        .split("://")
        .nth(1)
        .unwrap_or(&state.config.public_url)
        .split(':')
        .next()
        .unwrap_or("localhost");

    Ok(Json(json!({
        "challenge": challenge,
        "rp": {
            "name": "Noro Network",
            "id": domain
        },
        "user": {
            "id": u.id,
            "name": u.username,
            "displayName": u.username
        },
        "pubKeyCredParams": [
            { "type": "public-key", "alg": -7 },  // ES256
            { "type": "public-key", "alg": -257 } // RS256
        ],
        "authenticatorSelection": {
            "userVerification": "preferred"
        },
        "timeout": 60000
    })))
}

#[derive(Deserialize)]
pub struct RegisterVerifyReq {
    pub challenge: String,
    pub name: String,
    pub credential_id: String,
    pub public_key: String,
}

/// Сохранение зарегистрированного Passkey
pub async fn register_verify(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<RegisterVerifyReq>,
) -> AppResult<Json<Value>> {
    let challenge_user = crate::db::verify_and_consume_challenge(&state.db, &req.challenge)
        .await?
        .ok_or_else(|| AppError::BadRequest("Срок действия испытания истёк".into()))?;

    if challenge_user != Some(user.user_id) {
        return Err(AppError::Forbidden("Недействительный пользователь для испытания".into()));
    }

    let passkey = crate::db::create_passkey(
        &state.db,
        user.user_id,
        &req.name,
        &req.credential_id,
        &req.public_key,
    )
    .await?;

    Ok(Json(json!(passkey)))
}

/// Получение списка ключей пользователя
pub async fn list_passkeys(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let passkeys = crate::db::list_passkeys_for_user(&state.db, user.user_id).await?;
    Ok(Json(json!(passkeys)))
}

/// Удаление ключа пользователя
pub async fn delete_passkey(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let ok = crate::db::delete_passkey(&state.db, id, user.user_id).await?;
    Ok(Json(json!({ "success": ok })))
}

/// Вход: Генерация испытания для входа по Passkey
pub async fn login_options(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let challenge = random_challenge();
    crate::db::save_challenge(&state.db, &challenge, None).await?;

    let domain = state
        .config
        .public_url
        .split("://")
        .nth(1)
        .unwrap_or(&state.config.public_url)
        .split(':')
        .next()
        .unwrap_or("localhost");

    Ok(Json(json!({
        "challenge": challenge,
        "rpId": domain,
        "userVerification": "preferred",
        "timeout": 60000
    })))
}

#[derive(Deserialize)]
pub struct LoginVerifyReq {
    pub challenge: String,
    pub credential_id: String,
}

/// Вход: Проверка подписи Passkey и выдача авторизационного токена
pub async fn login_verify(
    State(state): State<AppState>,
    Json(req): Json<LoginVerifyReq>,
) -> AppResult<Json<Value>> {
    let valid_challenge = crate::db::verify_and_consume_challenge(&state.db, &req.challenge)
        .await?
        .is_some();

    if !valid_challenge {
        return Err(AppError::BadRequest("Срок действия испытания истёк".into()));
    }

    let passkey = crate::db::get_passkey_by_credential_id(&state.db, &req.credential_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Passkey не найден".into()))?;

    let session = crate::db::create_session(&state.db, passkey.user_id, "master", chrono::Duration::days(30)).await?;
    let user_profile = crate::db::load_profile(&state.db, passkey.user_id).await?;

    Ok(Json(json!({
        "access_token": session.access_token.to_string(),
        "user": user_profile
    })))
}
