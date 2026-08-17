//! Публичные настройки инстанса (название, hero-иллюстрация главной страницы).

use crate::config::keys;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

pub async fn get_public_settings(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let stored = crate::db::all_settings(&state.db).await?;

    let hero_image_url = crate::config::env_opt(keys::HERO_IMAGE_URL.env)
        .or_else(|| stored.get(keys::HERO_IMAGE_URL.name).and_then(|v| v.as_str()).map(|s| s.to_string()))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "/hero-character.jpg".to_string());

    let instance_name = crate::config::env_opt(keys::INSTANCE_NAME.env)
        .or_else(|| stored.get(keys::INSTANCE_NAME.name).and_then(|v| v.as_str()).map(|s| s.to_string()))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Noro Launcher".to_string());

    Ok(Json(json!({
        "hero_image_url": hero_image_url,
        "instance_name": instance_name,
    })))
}
