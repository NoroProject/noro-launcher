//! Свои сессии в кабинете.
//!
//! Отозвать скомпрометированный токен раньше было нечем: строка жила до
//! истечения срока, и «выйти на всех устройствах» означало ждать месяц.

use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

/// Bearer текущего запроса — чтобы пометить «эта сессия».
fn current_token(headers: &HeaderMap) -> Option<Uuid> {
    let raw = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?
        .trim();
    Uuid::parse_str(raw).ok()
}

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
) -> AppResult<Json<Value>> {
    let rows = crate::db::list_sessions(&state.db, user.user_id, current_token(&headers)).await?;
    Ok(Json(json!(rows)))
}

pub async fn revoke(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    if !crate::db::revoke_session(&state.db, id, Some(user.user_id)).await? {
        return Err(AppError::NotFound("session".into()));
    }
    Ok(Json(json!({ "ok": true })))
}

/// Завершить все, кроме текущей: иначе игрок выкидывает сам себя и не понимает,
/// почему страница вдруг разлогинилась.
pub async fn revoke_others(
    State(state): State<AppState>,
    user: AuthUser,
    headers: HeaderMap,
) -> AppResult<Json<Value>> {
    let revoked =
        crate::db::revoke_all_sessions(&state.db, user.user_id, current_token(&headers)).await?;
    Ok(Json(json!({ "revoked": revoked })))
}

/// GET /api/me/activity-heatmap — карточки активности игрока за год.
pub async fn activity_heatmap(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Value>> {
    let days = crate::db::game_sessions::get_activity_heatmap(&state.db, user.user_id).await?;
    Ok(Json(json!(days)))
}
