//! Админ: серверы.

use crate::api::auth::AdminAuth;
use crate::audit::{self, target};
use crate::db::models::ServerRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, Path, State};
use axum::Json;
use schema::PERM_ADMIN_SERVERS;
use serde::Deserialize;
use uuid::Uuid;

enum AssetKind {
    Icon,
    Background,
}

impl AssetKind {
    fn column(&self) -> &'static str {
        match self {
            Self::Icon => "icon_url",
            Self::Background => "background_url",
        }
    }
}

fn optimize_for_launcher(data: &[u8], kind: &AssetKind) -> Result<Vec<u8>, image::ImageError> {
    let img = image::load_from_memory(data)?;
    let (max_w, max_h) = match kind {
        AssetKind::Icon => (256, 256),
        AssetKind::Background => (1920, 1080),
    };
    let img = if img.width() > max_w || img.height() > max_h {
        img.resize(max_w, max_h, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let mut out = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut out);
    match kind {
        AssetKind::Icon => {
            img.write_to(&mut cursor, image::ImageFormat::WebP)?;
        }
        AssetKind::Background => {
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 85);
            img.write_with_encoder(encoder)?;
        }
    }
    Ok(out)
}

fn broadcast_servers_changed(state: &AppState) {
    state.ws.broadcast(&schema::ServerWsMsg::ServersChanged);
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Vec<ServerRow>>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    Ok(Json(crate::db::list_servers(&state.db, false).await?))
}

#[derive(Deserialize)]
pub struct CreateReq {
    pub name: String,
    pub modloader: String,
    pub mc_version: String,
    pub build_version: Option<String>,
    pub modloader_version: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<CreateReq>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_ADMIN_SERVERS)?;

    let mut tx = state.db.begin().await?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO servers (name, modloader, mc_version) VALUES ($1,$2,$3) RETURNING id",
    )
    .bind(&req.name)
    .bind(&req.modloader)
    .bind(&req.mc_version)
    .fetch_one(&mut *tx)
    .await?;

    if let Some(bv) = req.build_version {
        sqlx::query(
            "INSERT INTO builds (server_id, version, modloader, modloader_version, mc_version)
             VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(id)
        .bind(bv)
        .bind(&req.modloader)
        .bind(req.modloader_version)
        .bind(&req.mc_version)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    broadcast_servers_changed(&state);

    Ok(Json(serde_json::json!({ "id": id })))
}

#[derive(Deserialize)]
pub struct UpdateReq {
    pub name: String,
    pub description: String,
    pub modloader: String,
    pub mc_version: String,
    pub active: bool,
    pub limited: bool,
    pub sort_order: i32,
}

pub async fn update(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateReq>,
) -> AppResult<Json<ServerRow>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    sqlx::query(
        "UPDATE servers SET name=$2, description=$3, modloader=$4,
         mc_version=$5, active=$6, limited=$7, sort_order=$8 WHERE id=$1",
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.modloader)
    .bind(&req.mc_version)
    .bind(req.active)
    .bind(req.limited)
    .bind(req.sort_order)
    .execute(&state.db)
    .await?;
    let row = crate::db::get_server(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("сервер".into()))?;
    audit::record(
        &state,
        &admin.actor,
        "server.update",
        target("server", id),
        serde_json::json!({
            "name": req.name,
            "active": req.active,
            "limited": req.limited,
        }),
    )
    .await;
    broadcast_servers_changed(&state);
    Ok(Json(row))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    crate::db::delete_server(&state.db, id).await?;
    audit::record(
        &state,
        &admin.actor,
        "server.delete",
        target("server", id),
        serde_json::json!({}),
    )
    .await;
    broadcast_servers_changed(&state);
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Загрузить иконку/фон (multipart `image`), вернуть URL.
async fn upload_image(
    state: &AppState,
    id: Uuid,
    kind: AssetKind,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if field.name() == Some("image") {
            let raw = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            let column = kind.column();
            let data = tokio::task::spawn_blocking(move || optimize_for_launcher(&raw, &kind))
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
            let sql = format!("UPDATE servers SET {column} = $2 WHERE id = $1");
            sqlx::query(&sql)
                .bind(id)
                .bind(&url)
                .execute(&state.db)
                .await?;
            broadcast_servers_changed(state);
            return Ok(Json(serde_json::json!({ "url": url })));
        }
    }
    Err(AppError::BadRequest("нет поля image".into()))
}

pub async fn upload_icon(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    upload_image(&state, id, AssetKind::Icon, multipart).await
}

pub async fn upload_background(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    upload_image(&state, id, AssetKind::Background, multipart).await
}

#[derive(Deserialize)]
pub struct ReorderReq {
    pub order: Vec<Uuid>,
}

pub async fn reorder(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<ReorderReq>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    for (i, id) in req.order.iter().enumerate() {
        sqlx::query("UPDATE servers SET sort_order = $2 WHERE id = $1")
            .bind(id)
            .bind(i as i32)
            .execute(&state.db)
            .await?;
    }
    broadcast_servers_changed(&state);
    Ok(Json(serde_json::json!({ "ok": true })))
}
