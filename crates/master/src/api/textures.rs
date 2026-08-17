//! Встроенные текстуры, пресеты скинов и сервер рендеринга.

use super::skin_render;
use super::textures_source::{preset_bytes, resolve_skin_bytes, STEVE};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RenderQuery {
    pub mode: Option<String>,
    pub url: Option<String>,
    pub preset: Option<String>,
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub discord: Option<String>,
    pub scale: Option<u32>,
    pub overlay: Option<bool>,
    pub yaw: Option<f32>,
    pub pitch: Option<f32>,
    pub sway: Option<f32>,
}

pub async fn default_skin() -> Response {
    png(STEVE.to_vec(), "public, max-age=31536000, immutable")
}

pub async fn preset_skin_endpoint(Path(name): Path<String>) -> AppResult<Response> {
    let clean = name.trim_end_matches(".png").to_lowercase();
    // Неизвестный пресет — 404, а не молчаливый Стив: иначе опечатка в имени
    // выглядит как рабочая ссылка и живёт в вёрстке годами.
    let bytes =
        preset_bytes(&clean).ok_or_else(|| AppError::NotFound(format!("skin preset {clean}")))?;
    Ok(png(bytes.to_vec(), "public, max-age=31536000, immutable"))
}

pub async fn render_endpoint(
    State(state): State<AppState>,
    Query(q): Query<RenderQuery>,
) -> AppResult<Response> {
    let skin_bytes = resolve_skin_bytes(&state, &q).await?;
    // Скачали, но это не картинка — ошибка на нашей стороне или у источника.
    // Подменять её Стивом значит списать битую текстуру на «у игрока нет скина».
    let skin = image::load_from_memory(&skin_bytes)
        .map_err(|e| AppError::BadRequest(format!("the skin is not a readable image: {e}")))?;

    let scale = q.scale.unwrap_or(10).clamp(1, 64);
    let overlay = q.overlay.unwrap_or(true);
    let mode = q.mode.as_deref().unwrap_or("bust");
    let yaw = q.yaw.unwrap_or(-25.0);
    let pitch = q.pitch.unwrap_or(12.0);
    let sway = q.sway.unwrap_or(0.0);

    let bytes = match mode {
        "flat-head" | "flat_head" => skin_render::render_head(&skin, scale, overlay),
        "flat-bust" | "flat_bust" => skin_render::render_bust(&skin, scale, overlay),
        "flat-body" | "flat_body" => skin_render::render_body(&skin, scale, overlay),
        "cape" => skin_render::render_cape(&skin, scale),
        _ => super::skin_render_3d::render_3d(&skin, scale, overlay, mode, yaw, pitch, sway),
    };

    Ok(png(bytes, "public, max-age=3600"))
}

pub async fn render_head_endpoint(
    state: State<AppState>,
    Query(mut q): Query<RenderQuery>,
) -> AppResult<Response> {
    q.mode = Some("head".into());
    render_endpoint(state, Query(q)).await
}

pub async fn render_bust_endpoint(
    state: State<AppState>,
    Query(mut q): Query<RenderQuery>,
) -> AppResult<Response> {
    q.mode = Some("bust".into());
    render_endpoint(state, Query(q)).await
}

pub async fn render_body_endpoint(
    state: State<AppState>,
    Query(mut q): Query<RenderQuery>,
) -> AppResult<Response> {
    q.mode = Some("body".into());
    render_endpoint(state, Query(q)).await
}

pub async fn render_cape_endpoint(
    state: State<AppState>,
    Query(mut q): Query<RenderQuery>,
) -> AppResult<Response> {
    q.mode = Some("cape".into());
    render_endpoint(state, Query(q)).await
}

fn png(bytes: Vec<u8>, cache: &'static str) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, cache),
        ],
        bytes,
    )
        .into_response()
}
