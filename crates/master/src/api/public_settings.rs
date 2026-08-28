//! Instance settings the site needs before anyone signs in: name and artwork.

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
        // Empty means the site falls back to the icon bundled with its build.
        "logo_url": resolve(&stored, keys::LOGO_URL).unwrap_or_default(),
        "hero_image_url": value(keys::HERO_IMAGE_URL, "/hero-character.jpg"),
        // A transparent render goes straight onto the background with no
        // framed backdrop — there are no edges to outline.
        "hero_image_transparent": image_prep::transparency(&state, &stored, keys::HERO_IMAGE_URL).await,
        // Empty means the login page draws its own placeholder. Deliberately a
        // separate image from the hero: that one is wide, this one is a narrow
        // full-height column.
        "login_image_url": resolve(&stored, keys::LOGIN_IMAGE_URL).unwrap_or_default(),
        "instance_name": value(keys::INSTANCE_NAME, "Noro Launcher"),
    })))
}

/// The usual precedence: env over database. An empty string counts as unset.
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
