//! Админ: версии лаунчера, проверка GitHub, сборка, деплой.

use crate::api::auth::AdminAuth;
use crate::db::models::LauncherVersionRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use schema::{LauncherVersion, PERM_ADMIN_LAUNCHER};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn list_versions(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Vec<LauncherVersionRow>>> {
    admin.require(PERM_ADMIN_LAUNCHER)?;
    Ok(Json(crate::db::list_launcher_versions(&state.db).await?))
}

/// Проверить последний релиз на GitHub.
pub async fn github_latest(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_LAUNCHER)?;
    let repo = state
        .config
        .github_repo
        .clone()
        .ok_or_else(|| AppError::BadRequest("NORO_GITHUB_REPO не задан".into()))?;
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let mut req = state.http().get(&url).header("User-Agent", "noro-master");
    if let Some(tok) = &state.config.github_token {
        req = req.bearer_auth(tok);
    }
    let resp: Value = req
        .send()
        .await
        .map_err(|e| AppError::Other(e.into()))?
        .json()
        .await
        .map_err(|e| AppError::Other(e.into()))?;
    Ok(Json(
        json!({ "tag": resp["tag_name"], "name": resp["name"], "published_at": resp["published_at"] }),
    ))
}

#[derive(Deserialize)]
pub struct BuildReq {
    pub tag: String,
}

/// Собрать лаунчер из тега (git checkout + cargo build + подпись + store).
pub async fn build(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<BuildReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_LAUNCHER)?;
    let job_id = crate::launcher_builder::start_build(&state, &req.tag)
        .await
        .map_err(AppError::Other)?;
    Ok(Json(json!({ "job_id": job_id })))
}

pub async fn build_log(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(job_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_LAUNCHER)?;
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT status, log FROM launcher_build_jobs WHERE id=$1")
            .bind(job_id)
            .fetch_optional(&state.db)
            .await?;
    match row {
        Some((status, log)) => Ok(Json(json!({ "status": status, "log": log }))),
        None => Err(AppError::NotFound("задача сборки".into())),
    }
}

/// Задеплоить версию: пометить current и разослать всем лаунчерам.
pub async fn deploy(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(version_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_LAUNCHER)?;
    let row = crate::db::set_current_launcher_version(&state.db, version_id)
        .await?
        .ok_or_else(|| AppError::NotFound("версия лаунчера".into()))?;

    let version = LauncherVersion {
        id: row.id,
        version: row.version.clone(),
        platform: row.platform.clone(),
        url: state.config.file_url(&row.file_sha1),
        sha256: row.sha256.clone(),
        signature: row.signature.clone(),
        is_current: true,
    };
    state
        .ws
        .broadcast(&schema::ServerWsMsg::LauncherUpdate { version });
    Ok(Json(json!({ "ok": true, "deployed": row.version })))
}
