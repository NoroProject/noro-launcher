//! Админ: новости.

use crate::api::auth::AdminAuth;
use crate::db::models::NewsRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, Path, State};
use axum::Json;
use schema::PERM_ADMIN_NEWS;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

fn broadcast_news_changed(state: &AppState) {
    state.ws.broadcast(&schema::ServerWsMsg::NewsChanged);
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Vec<NewsRow>>> {
    admin.require(PERM_ADMIN_NEWS)?;
    Ok(Json(crate::db::list_news(&state.db, 100).await?))
}

#[derive(Deserialize)]
pub struct CreateReq {
    pub title: String,
    pub body: String,
    pub preview_img_url: Option<String>,
    #[serde(default)]
    pub pinned: bool,
}

pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<CreateReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_NEWS)?;
    let id = crate::db::create_news(
        &state.db,
        &req.title,
        &req.body,
        req.preview_img_url.as_deref(),
        admin.user_id(),
        req.pinned,
    )
    .await?;
    broadcast_news_changed(&state);
    Ok(Json(json!({ "id": id })))
}

pub async fn update(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_NEWS)?;
    sqlx::query("UPDATE news SET title=$2, body=$3, preview_img_url=$4, pinned=$5 WHERE id=$1")
        .bind(id)
        .bind(&req.title)
        .bind(&req.body)
        .bind(&req.preview_img_url)
        .bind(req.pinned)
        .execute(&state.db)
        .await?;
    broadcast_news_changed(&state);
    Ok(Json(json!({ "ok": true })))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_NEWS)?;
    crate::db::delete_news(&state.db, id).await?;
    broadcast_news_changed(&state);
    Ok(Json(json!({ "ok": true })))
}

/// Превью новости: файл принимается напрямую, а не ссылкой.
/// Ссылка требовала где-то отдельно разместить картинку и переживала бы
/// смерть чужого хостинга; здесь файл ложится в тот же content-addressed стор,
/// что и остальные ассеты, и получает иммутабельный URL.
pub async fn upload_image(
    State(state): State<AppState>,
    admin: AdminAuth,
    mut multipart: Multipart,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_NEWS)?;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if field.name() != Some("image") {
            continue;
        }
        let raw = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        let data = tokio::task::spawn_blocking(move || fit_preview(&raw))
            .await
            .map_err(|e| AppError::Other(e.into()))?
            .map_err(|e| AppError::BadRequest(format!("invalid image: {e}")))?;
        let stored = state
            .files
            .put_bytes(&data)
            .await
            .map_err(AppError::Other)?;
        let url = if let Some(s3) = &state.config.s3 {
            crate::files::s3::put(state.http(), s3, &stored.sha1, &data)
                .await
                .map_err(AppError::Other)?
        } else {
            state.config.file_url(&stored.sha1)
        };
        return Ok(Json(json!({ "url": url })));
    }
    Err(AppError::BadRequest("нет поля image".into()))
}

/// Превью показывается карточкой, поэтому 1280px по ширине с запасом хватает.
fn fit_preview(data: &[u8]) -> Result<Vec<u8>, image::ImageError> {
    let img = image::load_from_memory(data)?;
    let img = if img.width() > 1280 {
        img.resize(1280, 1280, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let mut out = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut out);
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 85);
    img.write_with_encoder(encoder)?;
    Ok(out)
}
