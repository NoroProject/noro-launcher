//! Встроенные текстуры, пресеты скинов и сервер рендеринга.

use super::skin_render;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

const STEVE: &[u8] = include_bytes!("../../assets/steve.png");
const ALEX: &[u8] = include_bytes!("../../assets/alex.png");
const ARI: &[u8] = include_bytes!("../../assets/ari.png");
const ZURI: &[u8] = include_bytes!("../../assets/zuri.png");
const EFE: &[u8] = include_bytes!("../../assets/efe.png");
const MAKENA: &[u8] = include_bytes!("../../assets/makena.png");
const KAI: &[u8] = include_bytes!("../../assets/kai.png");
const SUNNY: &[u8] = include_bytes!("../../assets/sunny.png");
const NOOR: &[u8] = include_bytes!("../../assets/noor.png");

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
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/png"), (header::CACHE_CONTROL, "public, max-age=31536000, immutable")],
        STEVE,
    ).into_response()
}

pub async fn preset_skin_endpoint(Path(name): Path<String>) -> Response {
    let clean = name.trim_end_matches(".png").to_lowercase();
    let bytes = get_preset_bytes(&clean);
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/png"), (header::CACHE_CONTROL, "public, max-age=31536000, immutable")],
        bytes,
    ).into_response()
}

pub async fn render_endpoint(State(state): State<AppState>, Query(q): Query<RenderQuery>) -> Response {
    let skin_bytes = resolve_skin_bytes(&state, &q).await;
    let skin = image::load_from_memory(&skin_bytes).unwrap_or_else(|_| image::load_from_memory(STEVE).unwrap());

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

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/png"), (header::CACHE_CONTROL, "public, max-age=3600")],
        bytes,
    ).into_response()
}

pub async fn render_head_endpoint(state: State<AppState>, Query(mut q): Query<RenderQuery>) -> Response {
    q.mode = Some("head".into());
    render_endpoint(state, Query(q)).await
}

pub async fn render_bust_endpoint(state: State<AppState>, Query(mut q): Query<RenderQuery>) -> Response {
    q.mode = Some("bust".into());
    render_endpoint(state, Query(q)).await
}

pub async fn render_body_endpoint(state: State<AppState>, Query(mut q): Query<RenderQuery>) -> Response {
    q.mode = Some("body".into());
    render_endpoint(state, Query(q)).await
}

pub async fn render_cape_endpoint(state: State<AppState>, Query(mut q): Query<RenderQuery>) -> Response {
    q.mode = Some("cape".into());
    render_endpoint(state, Query(q)).await
}

async fn resolve_skin_bytes(state: &AppState, q: &RenderQuery) -> Vec<u8> {
    if let Some(p) = &q.preset {
        return get_preset_bytes(p).to_vec();
    }
    if let Some(u) = &q.url {
        if let Ok(resp) = reqwest::get(u).await {
            if let Ok(b) = resp.bytes().await {
                return b.to_vec();
            }
        }
    }
    if let Some(name) = q.username.as_deref().or(q.uuid.as_deref()).or(q.discord.as_deref()) {
        if let Ok(users) = crate::db::list_users(&state.db, 500, 0).await {
            for u in users {
                if u.mc_username.eq_ignore_ascii_case(name)
                    || u.mc_uuid.to_string() == name
                    || u.discord_username.eq_ignore_ascii_case(name)
                    || u.discord_id == name
                {
                    if let Some(s_url) = &u.skin_url {
                        if let Ok(resp) = reqwest::get(s_url).await {
                            if let Ok(b) = resp.bytes().await {
                                return b.to_vec();
                            }
                        }
                    }
                }
            }
        }
    }
    STEVE.to_vec()
}

fn get_preset_bytes(name: &str) -> &'static [u8] {
    match name {
        "alex" => ALEX,
        "ari" => ARI,
        "zuri" => ZURI,
        "efe" => EFE,
        "makena" => MAKENA,
        "kai" => KAI,
        "sunny" => SUNNY,
        "noor" => NOOR,
        _ => STEVE,
    }
}
