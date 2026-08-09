//! Админ-API файлов игрового сервера.
//!
//! Всё уходит враппером на машину сервера; проверку путей делает он. Право то
//! же, что у остального управления, — `noro.admin.wrapper`.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::state::AppState;
use crate::wrapper::fs_ops::{self, ServerOutcome};
use axum::extract::{Path, Query, State};
use axum::Json;
use schema::PERM_ADMIN_WRAPPER;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathQuery {
    #[serde(default = "root")]
    pub path: String,
}

fn root() -> String {
    ".".into()
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<PathQuery>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    Ok(Json(fs_ops::list(&state, id, &q.path).await?))
}

pub async fn read(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<PathQuery>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    Ok(Json(fs_ops::read(&state, id, &q.path).await?))
}

#[derive(Deserialize)]
pub struct WriteReq {
    pub path: String,
    pub content: String,
}

pub async fn write(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<WriteReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    Ok(Json(
        fs_ops::write(&state, id, &req.path, &req.content).await?,
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Query(q): Query<PathQuery>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    fs_ops::delete(&state, id, &q.path).await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct MkdirReq {
    pub path: String,
}

pub async fn mkdir(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<MkdirReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    fs_ops::mkdir(&state, id, &req.path).await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct ApplyReq {
    pub path: String,
    pub content: String,
    /// Куда ещё разложить тот же файл. Пустой список — только исходный сервер.
    #[serde(default)]
    pub targets: Vec<Uuid>,
}

/// Один конфиг на несколько серверов сборки.
///
/// Ради этого и затевалось управление «всеми серверами сразу»: правка в одном
/// месте вместо обхода машин руками.
pub async fn apply(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<ApplyReq>,
) -> AppResult<Json<Vec<ServerOutcome>>> {
    admin.require(PERM_ADMIN_WRAPPER)?;
    let mut servers = vec![id];
    // Исходный сервер мог попасть и в список целей — дважды писать незачем.
    servers.extend(req.targets.into_iter().filter(|t| *t != id));

    let op = fs_ops::write_op(&req.path, &req.content);
    Ok(Json(fs_ops::fan_out(&state, &servers, op).await))
}
