//! Админ: флаги целостности.

use crate::api::auth::AdminAuth;
use crate::audit::{self, target};
use crate::db::integrity::IntegrityFlagRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use schema::PERM_ADMIN_USERS;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListQuery {
    pub user_id: Option<Uuid>,
    /// Только неразобранные — то, что нужно смотреть.
    #[serde(default)]
    pub open: bool,
    pub limit: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<IntegrityFlagRow>>> {
    admin.require(PERM_ADMIN_USERS)?;
    let rows =
        crate::db::list_integrity_flags(&state.db, q.user_id, q.open, q.limit.unwrap_or(100))
            .await?;
    Ok(Json(rows))
}

/// Пометить разобранным.
pub async fn review(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_USERS)?;
    let ok = crate::db::review_integrity_flag(&state.db, id, admin.user_id()).await?;
    if !ok {
        return Err(AppError::NotFound("флаг не найден или уже разобран".into()));
    }
    audit::record(
        &state,
        &admin.actor,
        "integrity.review",
        target("integrity_flag", id),
        json!({}),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}
