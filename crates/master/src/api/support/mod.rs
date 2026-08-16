//! Приём и выдача бандлов логов.
//!
//! Санитизация делается дважды: в лаунчере перед отправкой и здесь. Второй
//! проход обязателен — клиент можно подменить, и логи от подменённого клиента
//! не должны попасть во вьювер вместе с токенами.

mod resanitize;

use crate::api::auth::{AdminAuth, AuthUser};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// Потолок на архив: бандл — это логи, а не диск игрока.
const MAX_BUNDLE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Deserialize)]
pub struct UploadQuery {
    pub server_id: Option<Uuid>,
    #[serde(default)]
    pub note: String,
}

/// Приём бандла от лаунчера. Игрок отправляет свои логи сам — кнопкой
/// «Сообщить о проблеме» либо после краша.
pub async fn upload(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<UploadQuery>,
    body: Bytes,
) -> AppResult<Json<Value>> {
    if body.is_empty() {
        return Err(AppError::BadRequest("пустой бандл".into()));
    }
    if body.len() > MAX_BUNDLE_BYTES {
        return Err(AppError::BadRequest(format!(
            "бандл больше {} МБ",
            MAX_BUNDLE_BYTES / 1024 / 1024
        )));
    }

    // Второй проход санитизации: клиента можно подменить, и логи от
    // подменённого клиента не должны попасть во вьювер вместе с токенами.
    let clean = resanitize::resanitize_zip(&body)
        .map_err(|e| AppError::BadRequest(format!("бандл не разобрать: {e}")))?;

    let stored = state
        .files
        .put_bytes(&clean)
        .await
        .map_err(AppError::Other)?;

    let id = crate::db::create_support_bundle(
        &state.db,
        user.user_id,
        q.server_id,
        q.note.chars().take(2000).collect::<String>().trim(),
        true,
        &stored.sha1,
        stored.size as i64,
    )
    .await?;

    tracing::info!(%id, user = %user.user_id, bytes = clean.len(), "принят бандл логов");
    Ok(Json(json!({ "id": id })))
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
}

/// Список бандлов для админки.
pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Value>> {
    admin.require(schema::PERM_ADMIN_SUPPORT_LOGS)?;
    let rows = crate::db::list_support_bundles(&state.db, q.user_id, q.limit.unwrap_or(50)).await?;
    Ok(Json(json!(rows)))
}

/// Скачать архив.
pub async fn download(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    admin.require(schema::PERM_ADMIN_SUPPORT_LOGS)?;
    let row = crate::db::get_support_bundle(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("бандл".into()))?;

    let bytes = tokio::fs::read(state.files.path_for(&row.file_sha1))
        .await
        .map_err(|e| AppError::Other(e.into()))?;

    Ok((
        [
            ("content-type", "application/zip".to_string()),
            (
                "content-disposition",
                format!("attachment; filename=\"noro-bundle-{id}.zip\""),
            ),
        ],
        bytes,
    ))
}

/// Свои бандлы в кабинете.
pub async fn my_bundles(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    let rows = crate::db::list_support_bundles(&state.db, Some(user.user_id), 50).await?;
    Ok(Json(json!(rows)))
}

/// Удалить свой бандл. Только добровольно отправленные: иначе принудительный
/// сбор логов (§7.2) не имел бы смысла.
pub async fn delete_mine(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let ok = crate::db::delete_support_bundle(&state.db, id, Some(user.user_id), true).await?;
    if !ok {
        return Err(AppError::NotFound(
            "бандл не найден или удалить его нельзя".into(),
        ));
    }
    Ok(Json(json!({ "ok": true })))
}
