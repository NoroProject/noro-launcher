//! Сигнал жизни от игрового сервера.

use crate::api::auth::AgentAuth;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct HeartbeatReq {
    pub online: u32,
    pub max_players: u32,
    /// Версия ядра/лоадера — видно в админке, помогает при разборе проблем.
    #[serde(default)]
    pub version: Option<String>,
}

/// Обновляет онлайн и время последней связи. Лаунчеру уходит `ServersChanged`,
/// чтобы список пересчитал сумму.
pub async fn heartbeat(
    State(state): State<AppState>,
    agent: AgentAuth,
    Json(req): Json<HeartbeatReq>,
) -> AppResult<Json<serde_json::Value>> {
    let was_live = agent.game_server.live();
    crate::db::touch_game_server(
        &state.db,
        agent.game_server.id,
        req.online as i32,
        req.max_players as i32,
        req.version.as_deref(),
    )
    .await?;

    // Рассылать на каждый heartbeat — это шторм из 120 сообщений в час на
    // сервер ради чисел, которые лаунчер и так перечитает при открытии списка.
    // Важен только переход «сервер ожил» — карточка должна перестать быть
    // серой сразу.
    if !was_live {
        state.ws.broadcast(&schema::ServerWsMsg::ServersChanged);
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}
