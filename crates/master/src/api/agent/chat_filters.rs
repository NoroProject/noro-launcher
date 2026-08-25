//! GET /api/agent/chat-filters & POST /api/agent/automod-triggers

use crate::api::auth::AgentAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ChatFilterResp {
    pub filter_type: String,
    pub mode: String,
    pub enabled: bool,
    pub rule_code: Option<String>,
    pub whitelist: Vec<String>,
    pub words: Vec<String>,
    pub threshold: f64,
    pub min_length: i32,
    pub max_messages: i32,
    pub window_secs: i32,
}

pub async fn list(
    State(state): State<AppState>,
    _agent: AgentAuth,
) -> AppResult<Json<Vec<ChatFilterResp>>> {
    let rows = sqlx::query(
        "SELECT filter_type, mode, enabled, rule_code, whitelist, words, threshold, min_length, max_messages, window_secs
         FROM chat_filters"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        rows.into_iter()
            .map(|r| ChatFilterResp {
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

#[derive(Deserialize)]
pub struct TriggerReq {
    pub player_uuid: Uuid,
    pub filter_type: String,
    pub mode: String,
    pub trigger_text: String,
}

pub async fn record_trigger(
    State(state): State<AppState>,
    agent: AgentAuth,
    Json(req): Json<TriggerReq>,
) -> AppResult<Json<Value>> {
    let user_row = crate::db::user_by_mc_uuid(&state.db, req.player_uuid).await?;
    let user_id = match user_row {
        // Сработал фильтр на игрока, которого мастер не знает: записать триггер
        // некуда. Отдельный код, а не 200 с `{"ok": false}`, — иначе агент не
        // отличает «записал» от «не записал», не читая тело.
        None => {
            return Err(AppError::coded(
                axum::http::StatusCode::NOT_FOUND,
                crate::error_codes::PLAYER_NOT_FOUND,
                "no such player on the master",
            ))
        }
        Some(u) => u.id,
    };

    sqlx::query(
        "INSERT INTO automod_triggers (user_id, game_server_id, filter_type, mode, trigger_text)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(agent.game_server.server_id)
    .bind(&req.filter_type)
    .bind(&req.mode)
    .bind(&req.trigger_text)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({ "ok": true })))
}
