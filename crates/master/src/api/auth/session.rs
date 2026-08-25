//! Жизнь сессии после входа: продление и выход. Общее для всех способов войти.

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use chrono::Duration;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct RefreshReq {
    pub refresh_token: String,
}

/// Обновить access-токен по refresh-токену.
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshReq>,
) -> AppResult<Json<Value>> {
    let rt = uuid::Uuid::parse_str(&req.refresh_token)
        .map_err(|_| AppError::Unauthorized("invalid refresh token".into()))?;
    let session = crate::db::refresh_session(&state.db, rt, Duration::days(30))
        .await?
        .ok_or_else(|| AppError::Unauthorized("refresh token not found".into()))?;
    Ok(Json(serde_json::json!({
        "access_token": session.access_token,
        "refresh_token": session.refresh_token,
        "expires_at": session.expires_at,
    })))
}

/// Выйти: удалить текущую сессию (Bearer access-токен).
pub async fn logout(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> AppResult<Json<Value>> {
    if let Some(tok) = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|v| uuid::Uuid::parse_str(v.trim()).ok())
    {
        crate::db::delete_session(&state.db, tok).await?;
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}
