//! Данные, которые получает стороннее приложение по токену игрока.
//!
//! Единственная дверь для чужих токенов: везде остальном `AuthUser` их не
//! принимает. Каждая ручка называет свой scope — приложение видит ровно то, что
//! игрок ему разрешил, и ни полем больше.

use crate::api::auth::AppAuth;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

/// GET /api/oauth/me — кто вошёл.
///
/// Объём ответа зависит от выданных scope'ов: одна ручка вместо трёх, потому
/// что «покажи профиль» — это один запрос в любой интеграции, а разложить его
/// по трём адресам значит заставить всех делать три.
pub async fn me(State(state): State<AppState>, app: AppAuth) -> AppResult<Json<Value>> {
    let (_, profile) = app.require(schema::SCOPE_IDENTITY)?;

    let mut out = json!({
        "id": profile.id,
        "uuid": profile.uuid,
        "username": profile.username,
        "avatar_url": profile.primary_identity().and_then(|i| i.avatar_url.clone()),
    });

    if app.has(schema::SCOPE_PROFILE) {
        out["roles"] = json!(profile
            .roles
            .iter()
            .map(|r| json!({ "id": r.id, "name": r.name }))
            .collect::<Vec<_>>());
        out["banned"] = json!(profile.banned);
        out["frozen"] = json!(profile.frozen);
        out["created_at"] = json!(profile.created_at);
    }
    if app.has(schema::SCOPE_SKINS) {
        out["skin_url"] = json!(profile.skin_url);
        out["skin_slim"] = json!(profile.skin_slim);
        out["cape_url"] = json!(profile.cape_url);
    }
    if app.has(schema::SCOPE_IDENTITIES) {
        out["identities"] = json!(profile
            .identities
            .iter()
            .map(|i| json!({
                "provider": i.provider,
                "username": i.username,
                "is_primary": i.is_primary,
                "linked_at": i.linked_at,
            }))
            .collect::<Vec<_>>());
    }
    // Сессию приложения незачем спрашивать о том, чего оно не получило: пустое
    // поле честнее отсутствующего ключа, но и его здесь нет — приложение и так
    // знает свои scope'ы из ответа `/oauth2/token`.
    let _ = &state;
    Ok(Json(out))
}

/// GET /api/oauth/me/punishments — активные наказания игрока.
pub async fn punishments(State(state): State<AppState>, app: AppAuth) -> AppResult<Json<Value>> {
    let (user_id, _) = app.require(schema::SCOPE_PUNISHMENTS)?;
    crate::db::expire_punishments(&state.db).await?;
    Ok(Json(json!(
        crate::db::list_punishments(&state.db, user_id).await?
    )))
}

/// GET /api/oauth/me/capes — плащи, доступные игроку.
pub async fn capes(State(state): State<AppState>, app: AppAuth) -> AppResult<Json<Value>> {
    let (user_id, _) = app.require(schema::SCOPE_CAPES)?;
    Ok(Json(json!(
        crate::db::list_capes_for_user(&state.db, user_id).await?
    )))
}

/// GET /api/oauth/me/journal — с какой сборкой игрок заходил.
pub async fn journal(State(state): State<AppState>, app: AppAuth) -> AppResult<Json<Value>> {
    let (user_id, _) = app.require(schema::SCOPE_JOURNAL)?;
    Ok(Json(json!(
        crate::db::list_play_sessions(&state.db, user_id, 50).await?
    )))
}

/// GET /api/oauth/servers — серверы проекта и онлайн.
///
/// Данные и так публичные, но scope нужен: приложение, которому игрок его не
/// давал, не должно молча получать наш список — «ничего страшного» на каждом
/// шаге и складывается в доступ, которого никто не выдавал.
pub async fn servers(State(state): State<AppState>, app: AppAuth) -> AppResult<Json<Value>> {
    app.require(schema::SCOPE_SERVERS)?;
    let Json(list) = crate::api::servers::list(State(state)).await?;
    Ok(Json(json!(list)))
}
