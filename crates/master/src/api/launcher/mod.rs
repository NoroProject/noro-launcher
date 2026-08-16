//! API лаунчера: раздача версий и установщиков, WebSocket с мастером.

mod messages;
mod news;
mod servers;
pub mod ws;

use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

pub use ws::ws_handler;

#[derive(Deserialize)]
pub struct VersionQuery {
    pub platform: Option<String>,
}

/// Текущая версия лаунчера для платформы.
pub async fn current_version(
    State(state): State<AppState>,
    Query(q): Query<VersionQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let platform = q
        .platform
        .unwrap_or_else(|| schema::current_platform().to_string());
    let row = crate::db::current_launcher_version(&state.db, &platform).await?;
    match row {
        Some(r) => Ok(Json(serde_json::json!({
            "version": r.version,
            "platform": r.platform,
            "url": state.config.file_url(&r.file_sha1),
            "sha256": r.sha256,
            "signature": r.signature,
        }))),
        None => Ok(Json(serde_json::json!(null))),
    }
}

/// Установщики под все платформы — для кнопки скачивания на сайте.
///
/// Отдаётся без авторизации: лаунчер качают до того, как заводят аккаунт.
pub async fn downloads(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let rows = crate::db::current_bootstrappers(&state.db).await?;
    let items: Vec<_> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "platform": r.platform,
                "version": r.version,
                "size": r.size,
                "sha256": r.sha256,
                "url": state.config.file_url(&r.file_sha1),
                "filename": filename_for(&r.platform),
            })
        })
        .collect();
    Ok(Json(serde_json::json!(items)))
}

/// Имя файла для сохранения: стор адресуется по хешу и своего имени не знает.
fn filename_for(platform: &str) -> String {
    match platform {
        p if p.starts_with("windows") => "NoroLauncher.exe".into(),
        p if p.starts_with("macos") => "NoroLauncher.dmg".into(),
        _ => "noro-launcher".into(),
    }
}
