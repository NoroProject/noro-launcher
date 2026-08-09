//! Админ-API бэкапов игрового сервера.
//!
//! Создание, просмотр, восстановление и удаление zip-снимков директории сервера.
//! Все операции закрыты правом `noro.admin.wrapper`.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::state::AppState;
use crate::wrapper::ops;
use axum::extract::{Path, State};
use axum::Json;
use schema::PERM_ADMIN_WRAPPER;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
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
    admin.require(PERM_ADMIN_WRAPPER)?;
    Ok(Json(ops::backup_create(&state, id, &req.name).await?))
}

pub async fn restore(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, name)): Path<(Uuid, String)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    Ok(Json(ops::backup_restore(&state, id, &name).await?))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, name)): Path<(Uuid, String)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    ops::backup_delete(&state, id, &name).await?;
    Ok(Json(json!({ "ok": true })))
}
