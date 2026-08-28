//! Catalogue of permission nodes an agent sees on its own server.
//!
//! There is no way to know them up front: nodes come from the mods, and every
//! modpack has a different set. The agent reports whatever
//! `PermissionGatherEvent` handed it, and the admin panel autocompletes from
//! that rather than from a hardcoded list.
//!
//! Coverage is incomplete by nature — only mods that register through
//! `PermissionAPI` show up here. A mod that checks a permission string
//! directly never will.

use crate::api::auth::AgentAuth;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;

/// Sanity ceiling: 150 mods produce hundreds of nodes, not tens of thousands.
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
