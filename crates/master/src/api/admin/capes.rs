//! Admin cape catalog.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, Path, State};
use axum::Json;
use bytes::Bytes;
use schema::{CapeRow, PERM_CAPES_EDIT, PERM_CAPES_VIEW};

use uuid::Uuid;

const MAX_CAPE_BYTES: usize = 512 * 1024;

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Vec<CapeRow>>> {
    admin.require(PERM_CAPES_VIEW)?;
    Ok(Json(crate::db::list_capes(&state.db).await?))
}

pub async fn upload(
    State(state): State<AppState>,
    admin: AdminAuth,
    mut multipart: Multipart,
) -> AppResult<Json<CapeRow>> {
    admin.require(PERM_CAPES_EDIT)?;
    let mut name: Option<String> = None;
    let mut data: Option<Bytes> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        match field.name() {
            Some("name") => {
                name = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?,
                );
            }
            Some("cape") => {
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
    let name = validate_name(name)?;
    let data = validate_png(data)?;
    if crate::db::cape_name_exists(&state.db, &name).await? {
        return Err(AppError::Conflict("cape name already exists".into()));
    }
    let stored = state
        .files
        .put_bytes(&data)
        .await
        .map_err(AppError::Other)?;
    let url = state.config.file_url(&stored.sha1);
    Ok(Json(
        crate::db::insert_cape(
            &state.db,
            &name,
            &url,
            &stored.sha1,
            stored.size as i64,
            admin.user_id(),
        )
        .await?,
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CAPES_EDIT)?;
    if !crate::db::delete_cape(&state.db, id).await? {
        return Err(AppError::NotFound("cape".into()));
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

fn validate_name(name: Option<String>) -> AppResult<String> {
    let name = name.unwrap_or_default().trim().to_string();
    if !(2..=48).contains(&name.len()) {
        return Err(AppError::BadRequest("cape name must be 2-48 chars".into()));
    }
    Ok(name)
}

fn validate_png(data: Option<Bytes>) -> AppResult<Bytes> {
    let data = data.ok_or_else(|| AppError::BadRequest("missing cape field".into()))?;
    if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(AppError::BadRequest("PNG expected".into()));
    }
    if data.len() > MAX_CAPE_BYTES {
        return Err(AppError::BadRequest("cape is too large".into()));
    }
    Ok(data)
}
