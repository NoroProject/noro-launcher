//! Админ: база запрещённых файлов.

use crate::api::auth::AdminAuth;
use crate::audit;
use crate::db::blocklist::BlockedFileRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use schema::PERM_ADMIN_BUILDS;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Vec<BlockedFileRow>>> {
    admin.require(PERM_ADMIN_BUILDS)?;
    Ok(Json(crate::db::list_blocked_files(&state.db).await?))
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
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_BUILDS)?;

    let pattern = req
        .pattern
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty());
    let sha1 = req
        .sha1
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty());
    // Пустое правило матчило бы всё подряд и снесло игроку каталог.
    if pattern.is_none() && sha1.is_none() {
        return Err(AppError::BadRequest("нужна маска либо sha1".into()));
    }
    if let Some(h) = &sha1 {
        if h.len() != 40 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AppError::BadRequest("sha1 — это 40 hex-символов".into()));
        }
    }
    if req.reason.trim().len() < 3 {
        return Err(AppError::BadRequest(
            "нужна причина: она попадёт админу во флаг".into(),
        ));
    }

    let action = req.action.as_deref().unwrap_or("delete");
    if !matches!(action, "delete" | "flag" | "block_launch") {
        return Err(AppError::BadRequest("неизвестное действие".into()));
    }

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
        "blocklist.add",
        audit::target("blocked_file", row.id),
        json!({ "pattern": row.pattern, "sha1": row.sha1, "action": row.action }),
    )
    .await;

    Ok(Json(json!(row)))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_BUILDS)?;
    if !crate::db::delete_blocked_file(&state.db, id).await? {
        return Err(AppError::NotFound("правило".into()));
    }
    audit::record(
        &state,
        &admin.actor,
        "blocklist.remove",
        audit::target("blocked_file", id),
        json!({}),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}
