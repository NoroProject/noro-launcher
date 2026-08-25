//! Админ: база запрещённых файлов.

use crate::api::auth::AdminAuth;
use crate::api::created::created;
use crate::api::paging::Page;
use crate::api::validate::Validation;
use crate::audit;
use crate::db::blocklist::BlockedFileRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::Response;
use axum::Json;
use schema::{PERM_BLOCKLIST_EDIT, PERM_BLOCKLIST_VIEW};

use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Page<BlockedFileRow>>> {
    admin.require(PERM_BLOCKLIST_VIEW)?;
    Ok(Json(Page::whole(
        crate::db::list_blocked_files(&state.db).await?,
    )))
}

#[derive(Deserialize)]
pub struct CreateReq {
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
    pub reason: String,
    /// `delete` | `flag` | `block_launch`
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub server_id: Option<Uuid>,
}

pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<CreateReq>,
) -> AppResult<Response> {
    admin.require(PERM_BLOCKLIST_EDIT)?;

    let pattern = req
        .pattern
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty());
    let sha1 = req
        .sha1
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());
    let action = req.action.as_deref().unwrap_or("delete");

    Validation::new()
        // Пустое правило матчило бы всё подряд и снесло игроку каталог.
        .any_of("pattern", pattern.is_some() || sha1.is_some())
        .rule(
            "sha1",
            sha1.as_ref()
                .is_none_or(|h| h.len() == 40 && h.chars().all(|c| c.is_ascii_hexdigit())),
            "invalid_format",
            "sha1 is 40 hex characters",
        )
        .rule(
            "reason",
            req.reason.trim().chars().count() >= 3,
            "too_short",
            "a reason is required: it lands in the flag an admin will read",
        )
        .one_of("action", action, &["delete", "flag", "block_launch"])
        .finish()?;

    let row = crate::db::create_blocked_file(
        &state.db,
        pattern.as_deref(),
        sha1.as_deref(),
        req.reason.trim(),
        action,
        req.server_id,
        admin.user_id(),
    )
    .await?;

    audit::record(
        &state,
        &admin.actor,
        audit::actions::BLOCKLIST_ADD,
        audit::target("blocked_file", row.id),
        json!({ "pattern": row.pattern, "sha1": row.sha1, "action": row.action }),
    )
    .await;

    Ok(created(
        format!("/api/admin/blocklist/{}", row.id),
        json!(row),
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_BLOCKLIST_EDIT)?;
    if !crate::db::delete_blocked_file(&state.db, id).await? {
        return Err(AppError::NotFound("rule".into()));
    }
    audit::record(
        &state,
        &admin.actor,
        audit::actions::BLOCKLIST_REMOVE,
        audit::target("blocked_file", id),
        json!({}),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}
