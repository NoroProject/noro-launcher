//! Вход по recovery-коду.
//!
//! Это не «аварийный вариант». WebAuthn не работает по `http://` на
//! не-localhost адресе, то есть инстанс на `http://<ip>:8080` — типовой первый
//! запуск — не даёт привязать passkey вообще. Пароля в модели входа нет,
//! поэтому до настройки домена и TLS коды остаются единственным путём внутрь.
//!
//! Отсюда и решение: вход по коду даёт полноценную сессию, а не урезанную.

use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub code: String,
}

/// Войти по одноразовому коду.
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> AppResult<Json<Value>> {
    let user_id = crate::db::consume_recovery_code(&state.db, req.username.trim(), req.code.trim())
        .await?
        .ok_or_else(|| {
            // Не различаем «нет такого аккаунта» и «код не подошёл»: иначе форма
            // входа становится способом проверять существование аккаунтов.
            AppError::Unauthorized("Wrong username or code".into())
        })?;

    let left = crate::db::recovery_codes_left(&state.db, user_id).await?;
    let session =
        crate::db::create_session(&state.db, user_id, "master", chrono::Duration::days(30)).await?;
    let profile = crate::db::load_profile(&state.db, user_id).await?;

    crate::audit::record(
        &state,
        &crate::audit::Actor::User {
            id: user_id,
            username: profile.username.clone(),
        },
        crate::audit::actions::AUTH_RECOVERY_CODE,
        crate::audit::target("user", user_id),
        json!({ "codes_left": left }),
    )
    .await;
    tracing::warn!(%user_id, left, "вход по recovery-коду");

    Ok(Json(json!({
        "access_token": session.access_token.to_string(),
        "user": profile,
        "codes_left": left,
        // Экран «привяжите passkey» показывается сразу: код сгорел, и следующий
        // вход должен опираться на что-то более долговечное.
        "bind_passkey": true,
    })))
}

/// Перевыпустить коды. Старые сгорают разом — иначе «перевыпуск» означал бы
/// удвоение числа рабочих ключей от аккаунта.
pub async fn reissue(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let codes = crate::db::issue_recovery_codes(&state.db, user.user_id).await?;

    crate::audit::record(
        &state,
        &crate::audit::Actor::User {
            id: user.user_id,
            username: user.profile.username.clone(),
        },
        crate::audit::actions::AUTH_RECOVERY_REISSUE,
        crate::audit::target("user", user.user_id),
        json!({ "count": codes.len() }),
    )
    .await;

    Ok(Json(json!({ "codes": codes })))
}

/// Сколько кодов осталось — для предупреждения в кабинете.
pub async fn remaining(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let left = crate::db::recovery_codes_left(&state.db, user.user_id).await?;
    Ok(Json(json!({
        "left": left,
        // Три — порог, после которого стоит перевыпустить, пока вход ещё есть.
        "low": left <= 3,
    })))
}
