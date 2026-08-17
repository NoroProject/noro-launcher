//! Админ: чтение журнала действий.

use crate::api::auth::AdminAuth;
use crate::db::audit::{AuditFilter, AuditRow};
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use schema::PERM_AUDIT;

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

/// Справочник для фильтра: какие события бывают и как они называются.
///
/// Реестр из кода, а не `SELECT DISTINCT`: иначе в списке не было бы событий,
/// которые ещё ни разу не случились, — а искать чаще всего надо именно их.
pub async fn actions(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_AUDIT)?;

    // Что реально встречается в журнале — чтобы старые события, выпавшие из
    // реестра, не пропали из фильтра молча.
    let seen = crate::db::distinct_audit_actions(&state.db).await?;

    let mut items: Vec<serde_json::Value> = crate::audit::actions::ALL
        .iter()
        .map(|a| serde_json::json!({ "name": a.name, "group": a.group, "title": a.title }))
        .collect();
    for name in seen {
        if !crate::audit::actions::ALL.iter().any(|a| a.name == name) {
            items.push(serde_json::json!({
                "name": name,
                "group": "Other",
                "title": name,
            }));
        }
    }

    Ok(Json(serde_json::json!({
        "actions": items,
        "groups": crate::audit::actions::GROUPS,
        "target_kinds": ["user", "role", "build", "server", "admin_token",
                         "launcher_version", "blocked_file", "integrity_flag"],
    })))
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<AuditRow>>> {
    admin.require(PERM_AUDIT)?;
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
