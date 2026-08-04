//! Админ: серверные ядра (server.jar) для автопатча/деплоя серверов.

use crate::api::auth::AdminAuth;
use crate::db::models::ServerCoreRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, Path, Query, State};
use axum::Json;
use schema::PERM_ADMIN_SERVERS;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListQuery {
    pub server_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct ServerCoreResp {
    #[serde(flatten)]
    pub row: ServerCoreRow,
    pub url: String,
}

fn with_url(state: &AppState, row: ServerCoreRow) -> ServerCoreResp {
    ServerCoreResp {
        url: state.config.file_url(&row.file_sha1),
        row,
    }
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<ServerCoreResp>>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    let rows = crate::db::list_server_cores(&state.db, q.server_id).await?;
    Ok(Json(
        rows.into_iter().map(|row| with_url(&state, row)).collect(),
    ))
}

/// Multipart upload: `file`, `server_id`, optional `version`.
pub async fn upload(
    State(state): State<AppState>,
    admin: AdminAuth,
    mut multipart: Multipart,
) -> AppResult<Json<ServerCoreResp>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    let mut server_id: Option<Uuid> = None;
    let mut version: Option<String> = None;
    let mut data: Option<bytes::Bytes> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        match field.name() {
            Some("server_id") => {
                let raw = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                server_id = Some(
                    raw.parse()
                        .map_err(|_| AppError::BadRequest("server_id должен быть UUID".into()))?,
                );
            }
            Some("version") => {
                version = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?,
                );
            }
            Some("file") => {
                filename = field.file_name().map(String::from);
                data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?,
                );
            }
            _ => {}
        }
    }

    let server_id = server_id.ok_or_else(|| AppError::BadRequest("нет поля server_id".into()))?;
    let data = data.ok_or_else(|| AppError::BadRequest("нет поля file".into()))?;
    let version = version
        .filter(|v| !v.trim().is_empty())
        .or(filename)
        .unwrap_or_else(|| "server.jar".into());

    let sha256 = crate::files::store::sha256_bytes(&data);
    let stored = state
        .files
        .put_bytes(&data)
        .await
        .map_err(AppError::Other)?;
    let id = crate::db::insert_server_core(
        &state.db,
        server_id,
        &version,
        &sha256,
        &stored.sha1,
        stored.size as i64,
    )
    .await?;
    let row = crate::db::list_server_cores(&state.db, Some(server_id))
        .await?
        .into_iter()
        .find(|row| row.id == id)
        .ok_or_else(|| AppError::NotFound("ядро сервера".into()))?;
    Ok(Json(with_url(&state, row)))
}

pub async fn activate(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ServerCoreResp>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    let row = crate::db::activate_server_core(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("ядро сервера".into()))?;
    Ok(Json(with_url(
        &state,
        ServerCoreRow {
            active: true,
            ..row
        },
    )))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    crate::db::delete_server_core(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
