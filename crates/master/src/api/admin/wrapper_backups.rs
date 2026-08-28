//! Zip snapshots of a game server directory: create, list, restore, delete.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::state::AppState;
use crate::wrapper::ops;
use axum::extract::{Path, State};
use axum::Json;
use schema::PERM_WRAPPER_BACKUPS;

use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_WRAPPER_BACKUPS)?;
    Ok(Json(ops::backup_list(&state, id).await?))
}

#[derive(Deserialize)]
pub struct CreateReq {
    #[serde(default)]
    pub name: String,
}

pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_WRAPPER_BACKUPS)?;
    Ok(Json(ops::backup_create(&state, id, &req.name).await?))
}

pub async fn restore(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, name)): Path<(Uuid, String)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_WRAPPER_BACKUPS)?;
    Ok(Json(ops::backup_restore(&state, id, &name).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, name)): Path<(Uuid, String)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_WRAPPER_BACKUPS)?;
    ops::backup_delete(&state, id, &name).await?;
    Ok(Json(json!({ "ok": true })))
}
