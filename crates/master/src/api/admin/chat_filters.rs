//! Admin endpoints for chat filters.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;

#[derive(Serialize, Deserialize)]
pub struct ChatFilterItem {
    pub filter_type: String,
    pub mode: String,
    pub enabled: bool,
    pub rule_code: Option<String>,
    #[serde(default)]
    pub whitelist: Vec<String>,
    #[serde(default)]
    pub words: Vec<String>,
    #[serde(default)]
    pub threshold: f64,
    #[serde(default)]
    pub min_length: i32,
    #[serde(default)]
    pub max_messages: i32,
    #[serde(default)]
    pub window_secs: i32,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Vec<ChatFilterItem>>> {
    admin.require(schema::PERM_PUNISH_MUTE)?;
    let rows = sqlx::query(
        "SELECT filter_type, mode, enabled, rule_code, whitelist, words, threshold, min_length, max_messages, window_secs
         FROM chat_filters"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| ChatFilterItem {
                filter_type: r.get("filter_type"),
                mode: r.get("mode"),
                enabled: r.get("enabled"),
                rule_code: r.get("rule_code"),
                whitelist: r.get("whitelist"),
                words: r.get("words"),
                threshold: r.get("threshold"),
                min_length: r.get("min_length"),
                max_messages: r.get("max_messages"),
                window_secs: r.get("window_secs"),
            })
            .collect(),
    ))
}

pub async fn save(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<ChatFilterItem>,
) -> AppResult<Json<Value>> {
    admin.require(schema::PERM_PUNISH_MUTE)?;
    sqlx::query(
        "INSERT INTO chat_filters (filter_type, mode, enabled, rule_code, whitelist, words, threshold, min_length, max_messages, window_secs)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (filter_type) DO UPDATE SET
            mode = EXCLUDED.mode,
            enabled = EXCLUDED.enabled,
            rule_code = EXCLUDED.rule_code,
            whitelist = EXCLUDED.whitelist,
            words = EXCLUDED.words,
            threshold = EXCLUDED.threshold,
            min_length = EXCLUDED.min_length,
            max_messages = EXCLUDED.max_messages,
            window_secs = EXCLUDED.window_secs,
            updated_at = NOW()"
    )
    .bind(&req.filter_type)
    .bind(&req.mode)
    .bind(req.enabled)
    .bind(&req.rule_code)
    .bind(&req.whitelist)
    .bind(&req.words)
    .bind(req.threshold)
    .bind(req.min_length)
    .bind(req.max_messages)
    .bind(req.window_secs)
    .execute(&state.db)
    .await?;

    crate::agent_link::notify::filters_changed(&state);
    Ok(Json(json!({ "ok": true })))
}
