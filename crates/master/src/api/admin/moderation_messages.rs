//! Админ: тексты, которые игрок видит при бане, муте и варне.

use crate::api::auth::AdminAuth;
use crate::api::moderation_messages::{ModerationMessages, SETTINGS_KEY};
use crate::audit;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::json;

/// Править тексты — это править то, что видит игрок при отказе, поэтому право
/// то же, что и на остальные настройки инстанса.
use schema::{PERM_SETTINGS_EDIT, PERM_SETTINGS_VIEW};

pub async fn get(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_SETTINGS_VIEW)?;
    Ok(Json(json!({
        "messages": crate::api::moderation_messages::load(&state.db).await,
        // Умолчания нужны панели, чтобы показать «вернуть как было» без
        // второго источника правды в вебе.
        "defaults": ModerationMessages::default(),
    })))
}

pub async fn put(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<ModerationMessages>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_SETTINGS_EDIT)?;

    // Пустой экран бана — это игрок, которого выкинуло без единого слова.
    // Пустым разрешено быть только объявлению: молчаливый сервер — рабочий
    // выбор, а молчаливый кик — нет.
    let value = serde_json::to_value(&req).map_err(|e| AppError::Other(e.into()))?;
    for (key, text) in value.as_object().into_iter().flatten() {
        if key != "broadcast" && text.as_str().unwrap_or_default().trim().is_empty() {
            return Err(AppError::BadRequest(format!("{key} cannot be empty")));
        }
    }

    crate::db::set_setting(&state.db, SETTINGS_KEY, &value, admin.user_id()).await?;
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
