//! Каталог узлов прав, которые агент видит на своём сервере.
//!
//! Перечислить их заранее неоткуда: узлы приносят сами моды, и у каждой сборки
//! свой набор. Поэтому список присылает агент — тем, что вернул ему
//! `PermissionGatherEvent`, — а админка потом подсказывает из реальных данных,
//! а не из захардкоженного перечня.
//!
//! Покрытие неполное по своей природе: сюда попадают только моды, которые
//! регистрируют узлы через `PermissionAPI`. Кто проверяет права строкой напрямую,
//! здесь не появится — и обещать обратное нельзя.

use crate::api::auth::AgentAuth;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;

/// Потолок на всякий случай: сборка в полторы сотни модов даёт сотни узлов,
/// но не десятки тысяч.
const MAX_NODES: usize = 5000;

#[derive(Deserialize)]
pub struct NodesReq {
    pub nodes: Vec<String>,
}

pub async fn report(
    State(state): State<AppState>,
    agent: AgentAuth,
    Json(req): Json<NodesReq>,
) -> AppResult<Json<serde_json::Value>> {
    let server_id = agent.game_server.server_id;

    let mut nodes: Vec<String> = req
        .nodes
        .into_iter()
        .map(|node| node.trim().to_string())
        .filter(|node| !node.is_empty() && node.len() <= 256)
        .collect();
    nodes.sort();
    nodes.dedup();
    nodes.truncate(MAX_NODES);

    let accepted = nodes.len();
    crate::db::replace_permission_nodes(&state.db, server_id, &nodes).await?;
    Ok(Json(
        serde_json::json!({ "ok": true, "accepted": accepted }),
    ))
}
