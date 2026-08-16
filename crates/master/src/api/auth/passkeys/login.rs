//! Вход по passkey — без ввода имени: кто входит, говорит сам ключ.

use super::register::encode_cred_id;
use super::{reject, ChallengeRes};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use webauthn_rs::prelude::*;

/// Опции для `navigator.credentials.get()`.
pub async fn login_options(
    State(state): State<AppState>,
) -> AppResult<Json<ChallengeRes<RequestChallengeResponse>>> {
    let (options, auth_state) = state
        .webauthn()?
        .start_discoverable_authentication()
        .map_err(reject)?;

    let state_id = crate::db::save_webauthn_state(
        &state.db,
        "login",
        None,
        &serde_json::to_value(&auth_state)?,
    )
    .await?;

    Ok(Json(ChallengeRes { state_id, options }))
}

#[derive(Deserialize)]
pub struct LoginVerifyReq {
    pub state_id: Uuid,
    pub credential: PublicKeyCredential,
}

/// Проверить подпись и сказать, чей это ключ.
///
/// `userHandle` из ответа — значение неподтверждённое, его подставляет клиент.
/// Доверять ему можно только потому, что подпись затем проверяется ключами
/// именно этого игрока: подставив чужой id, злоумышленник получит отказ.
pub async fn verify(state: &AppState, req: &LoginVerifyReq) -> AppResult<Uuid> {
    let (_, raw_state) = crate::db::take_webauthn_state(&state.db, req.state_id, "login")
        .await?
        .ok_or_else(|| AppError::BadRequest("Срок действия испытания истёк".into()))?;
    let auth_state: DiscoverableAuthentication = serde_json::from_value(raw_state)?;

    let (user_id, _) = state
        .webauthn()?
        .identify_discoverable_authentication(&req.credential)
        .map_err(reject)?;

    let rows = crate::db::passkeys_for_user(&state.db, user_id).await?;
    let mut keys: Vec<(Uuid, Passkey)> = Vec::with_capacity(rows.len());
    for r in rows {
        match serde_json::from_value::<Passkey>(r.credential) {
            Ok(pk) => keys.push((r.id, pk)),
            // Битую запись пропускаем, но молчать нельзя: у игрока внезапно
            // «перестал работать ключ», и причина должна быть в логе.
            Err(e) => tracing::error!(passkey = %r.id, error = %e, "не разобрать passkey из БД"),
        }
    }
    if keys.is_empty() {
        return Err(AppError::Unauthorized("Ключ не подошёл".into()));
    }

    let discoverable: Vec<DiscoverableKey> = keys.iter().map(|(_, pk)| pk.into()).collect();
    let result = state
        .webauthn()?
        .finish_discoverable_authentication(&req.credential, auth_state, &discoverable)
        .map_err(reject)?;

    // Счётчик растёт с каждым входом; его откат — признак клонированного ключа,
    // и решает это библиотека внутри проверки выше. Наше дело — сохранить новое
    // значение, иначе проверка перестаёт что-либо ловить.
    let used = encode_cred_id(result.cred_id());
    for (row_id, mut pk) in keys {
        if encode_cred_id(pk.cred_id()) == used {
            pk.update_credential(&result);
            crate::db::update_passkey_credential(&state.db, row_id, &serde_json::to_value(&pk)?)
                .await?;
            break;
        }
    }

    Ok(user_id)
}

/// Вход на сайте: сессия на 30 дней и профиль.
pub async fn login_verify(
    State(state): State<AppState>,
    Json(req): Json<LoginVerifyReq>,
) -> AppResult<Json<Value>> {
    let user_id = verify(&state, &req).await?;

    let session =
        crate::db::create_session(&state.db, user_id, "master", chrono::Duration::days(30)).await?;
    crate::audit::record_by_user(
        &state,
        user_id,
        "auth.login",
        serde_json::json!({ "method": "passkey" }),
    )
    .await;
    let user_profile = crate::db::load_profile(&state.db, user_id).await?;

    Ok(Json(json!({
        "access_token": session.access_token.to_string(),
        "user": user_profile
    })))
}
