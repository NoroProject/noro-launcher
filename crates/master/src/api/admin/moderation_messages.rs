//! The text a player sees on a ban, mute or warn.

use crate::api::auth::AdminAuth;
use crate::api::moderation_messages::{ModerationMessages, SETTINGS_KEY};
use crate::audit;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::json;

/// Guarded by the instance settings permissions rather than a moderation one:
/// this edits instance-wide copy, not any individual punishment.
use schema::{PERM_SETTINGS_EDIT, PERM_SETTINGS_VIEW};

pub async fn get(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_SETTINGS_VIEW)?;
    let map = crate::api::moderation_messages::load_map(&state.db).await;
    let mut defaults = std::collections::HashMap::new();
    defaults.insert("en", ModerationMessages::default_en());
    defaults.insert("ru", ModerationMessages::default_ru());

    let single_fallback = map
        .get("en")
        .cloned()
        .unwrap_or_else(ModerationMessages::default_en);
    Ok(Json(json!({
        "messages": single_fallback,
        "map": map,
        "defaults": ModerationMessages::default_en(),
        "defaults_map": defaults,
    })))
}

pub async fn put(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_SETTINGS_EDIT)?;

    if req.get("ban_permanent").is_some() {
        if let Ok(msg) = serde_json::from_value::<ModerationMessages>(req.clone()) {
            let mut map = crate::api::moderation_messages::load_map(&state.db).await;
            map.insert("en".to_string(), msg);
            let val = serde_json::to_value(&map).map_err(|e| AppError::Other(e.into()))?;
            crate::db::set_setting(&state.db, SETTINGS_KEY, &val, admin.user_id()).await?;
        }
    } else {
        crate::db::set_setting(&state.db, SETTINGS_KEY, &req, admin.user_id()).await?;
    }

    crate::agent_link::notify::messages_changed(&state);

    audit::record(
        &state,
        &admin.actor,
        audit::actions::SETTINGS_UPDATE,
        None,
        json!({ "section": SETTINGS_KEY }),
    )
    .await;

    Ok(Json(json!({ "ok": true })))
}
