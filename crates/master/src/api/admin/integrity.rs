//! Админ: флаги целостности.

use crate::api::auth::AdminAuth;
use crate::audit::{self, target};
use crate::db::integrity::IntegrityFlagRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use schema::{PERM_INTEGRITY_REVIEW, PERM_INTEGRITY_VIEW};

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
    admin.require(PERM_INTEGRITY_VIEW)?;
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
    admin.require(PERM_INTEGRITY_REVIEW)?;
    let ok = crate::db::review_integrity_flag(&state.db, id, admin.user_id()).await?;
    if !ok {
        return Err(AppError::NotFound(
            "flag not found or already reviewed".into(),
        ));
    }
    audit::record(
        &state,
        &admin.actor,
        audit::actions::INTEGRITY_REVIEW,
        target("integrity_flag", id),
        json!({}),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}
