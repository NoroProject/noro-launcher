//! Админ: чтение журнала действий.

use crate::api::auth::AdminAuth;
use crate::db::audit::{AuditFilter, AuditRow};
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use schema::PERM_ADMIN_AUDIT;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListQuery {
    pub actor_id: Option<Uuid>,
    /// Префикс: `user` покрывает `user.ban`, `user.role.add` и так далее.
    pub action: Option<String>,
    pub target_kind: Option<String>,
    pub target_id: Option<String>,
    /// Курсор: следующая страница — записи старше этого id.
    pub before_id: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<AuditRow>>> {
    admin.require(PERM_ADMIN_AUDIT)?;
    let filter = AuditFilter {
        actor_id: q.actor_id,
        action: q.action.filter(|s| !s.is_empty()),
        target_kind: q.target_kind.filter(|s| !s.is_empty()),
        target_id: q.target_id.filter(|s| !s.is_empty()),
        before_id: q.before_id,
    };
    let rows = crate::db::list_audit(&state.db, &filter, q.limit.unwrap_or(50)).await?;
    Ok(Json(rows))
}
