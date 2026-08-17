//! Админ: версии лаунчера, проверка GitHub, сборка, деплой.

use crate::api::auth::AdminAuth;
use crate::audit::{self, target};
use crate::db::models::LauncherVersionRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use schema::{LauncherVersion, PERM_LAUNCHER_DEPLOY, PERM_LAUNCHER_PUBLISH, PERM_LAUNCHER_VIEW};

use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn list_versions(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Vec<LauncherVersionRow>>> {
    admin.require(PERM_LAUNCHER_VIEW)?;
    Ok(Json(crate::db::list_launcher_versions(&state.db).await?))
}

/// Проверить последний релиз на GitHub.
pub async fn github_latest(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Value>> {
    admin.require(PERM_LAUNCHER_VIEW)?;
    let repo = state
        .config
        .github_repo
        .clone()
        .ok_or_else(|| AppError::BadRequest("NORO_GITHUB_REPO is not set".into()))?;
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
    admin.require(PERM_LAUNCHER_PUBLISH)?;
    let job_id = crate::launcher_builder::start_build(&state, &req.tag)
        .await
        .map_err(AppError::Other)?;
    Ok(Json(json!({ "job_id": job_id })))
}

/// Последние задачи сборки — чтобы job_id не приходилось откуда-то переписывать
/// руками: у админки не было списка, и лог прошлой сборки открыть было нечем.
pub async fn build_jobs(State(state): State<AppState>, admin: AdminAuth) -> AppResult<Json<Value>> {
    admin.require(PERM_LAUNCHER_PUBLISH)?;
    let rows: Vec<(Uuid, String, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT id, github_tag, status, created_at FROM launcher_build_jobs
         ORDER BY created_at DESC LIMIT 20",
    )
    .fetch_all(&state.db)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(id, tag, status, created_at)| {
            json!({
                "id": id,
                "tag": tag,
                "status": status,
                "created_at": created_at,
            })
        })
        .collect();
    Ok(Json(json!(items)))
}

pub async fn build_log(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(job_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_LAUNCHER_PUBLISH)?;
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT status, log FROM launcher_build_jobs WHERE id=$1")
            .bind(job_id)
            .fetch_optional(&state.db)
            .await?;
    match row {
        Some((status, log)) => Ok(Json(json!({ "status": status, "log": log }))),
        None => Err(AppError::NotFound("build job".into())),
    }
}

/// Задеплоить версию: пометить current и разослать всем лаунчерам.
pub async fn deploy(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(version_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_LAUNCHER_DEPLOY)?;
    let row = crate::db::set_current_launcher_version(&state.db, version_id)
        .await?
        .ok_or_else(|| AppError::NotFound("launcher version".into()))?;

    let version = LauncherVersion {
        id: row.id,
        version: row.version.clone(),
        platform: row.platform.clone(),
        url: state.config.file_url(&row.file_sha1),
        sha256: row.sha256.clone(),
        signature: row.signature.clone(),
        is_current: true,
    };
    audit::record(
        &state,
        &admin.actor,
        audit::actions::LAUNCHER_DEPLOY,
        target("launcher_version", version_id),
        json!({ "version": row.version, "platform": row.platform }),
    )
    .await;
    state
        .ws
        .broadcast(&schema::ServerWsMsg::LauncherUpdate { version });
    Ok(Json(json!({ "ok": true, "deployed": row.version })))
}
