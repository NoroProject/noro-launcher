//! Публичные настройки инстанса: название и иллюстрации, которые сайт
//! показывает ещё до входа.

use crate::api::admin::settings::image_prep;
use crate::config::keys::{self, Key};
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub async fn get_public_settings(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let stored = crate::db::all_settings(&state.db).await?;
    let value = |key: Key, default: &str| -> String {
        resolve(&stored, key).unwrap_or_else(|| default.to_string())
    };

    Ok(Json(json!({
        // Пусто — сайт рисует свой значок из сборки: инстанс без залитого лого
        // не должен остаться вовсе без шапки.
        "logo_url": resolve(&stored, keys::LOGO_URL).unwrap_or_default(),
        "hero_image_url": value(keys::HERO_IMAGE_URL, "/hero-character.jpg"),
        // Прозрачную иллюстрацию сайт кладёт на фон как есть, без подложки с
        // рамкой: у вырезанного рендера её края обводить нечего.
        "hero_image_transparent": image_prep::transparency(&state, &stored, keys::HERO_IMAGE_URL).await,
        // Пусто — страница входа рисует заглушку сама. Отдельная картинка, а
        // не та же, что на главной: там герой в полный рост под широкий блок,
        // здесь — узкая колонка во всю высоту экрана.
        "login_image_url": resolve(&stored, keys::LOGIN_IMAGE_URL).unwrap_or_default(),
        "instance_name": value(keys::INSTANCE_NAME, "Noro Launcher"),
    })))
}

/// Значение настройки по общему правилу: env > БД. Пустая строка — «не задано».
fn resolve(stored: &BTreeMap<String, Value>, key: Key) -> Option<String> {
    crate::config::env_opt(key.env)
        .or_else(|| {
            stored
                .get(key.name)
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .filter(|s| !s.trim().is_empty())
}
