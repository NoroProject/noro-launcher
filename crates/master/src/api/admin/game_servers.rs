//! Админ: игровые сервера сборки и их секреты.

use crate::api::auth::{generate_agent_secret, hash_agent_secret, AdminAuth};
use crate::api::paging::Page;
use crate::db::game_servers::GameServerRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use schema::PERM_SERVERS_AGENTS;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Serialize)]
pub struct GameServerResp {
    #[serde(flatten)]
    pub row: GameServerRow,
    pub live: bool,
}

fn default_port() -> i32 {
    25565
}

fn broadcast(state: &AppState) {
    state.ws.broadcast(&schema::ServerWsMsg::ServersChanged);
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(server_id): Path<Uuid>,
) -> AppResult<Json<Page<GameServerResp>>> {
    admin.require(PERM_SERVERS_AGENTS)?;
    let rows = crate::db::list_game_servers(&state.db, server_id).await?;
    Ok(Json(Page::whole(
        rows.into_iter()
            .map(|row| GameServerResp {
                live: row.live(),
                row,
            })
            .collect(),
    )))
}

#[derive(Deserialize)]
pub struct SaveReq {
    pub name: String,
    #[serde(default)]
    pub mc_host: String,
    #[serde(default = "default_port")]
    pub mc_port: i32,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub maintenance: bool,
    #[serde(default)]
    pub maintenance_reason: Option<String>,
    #[serde(default)]
    pub countdown_seconds: Option<u32>,
}

impl SaveReq {
    fn kind(&self) -> &str {
        match self.kind.as_deref() {
            Some("proxy") => "proxy",
            _ => "server",
        }
    }
}

pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(server_id): Path<Uuid>,
    Json(req): Json<SaveReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_SERVERS_AGENTS)?;
    if crate::db::get_server(&state.db, server_id).await?.is_none() {
        return Err(AppError::NotFound("build not found".into()));
    }
    let secret = generate_agent_secret();
    let row = crate::db::create_game_server(
        &state.db,
        server_id,
        req.name.trim(),
        req.mc_host.trim(),
        req.mc_port,
        &hash_agent_secret(&secret),
        req.kind(),
    )
    .await?;
    broadcast(&state);
    Ok(Json(json!({ "server": row, "secret": secret })))
}

pub async fn update(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((server_id, id)): Path<(Uuid, Uuid)>,
    Json(req): Json<SaveReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_SERVERS_AGENTS)?;
    crate::db::update_game_server(
        &state.db,
        id,
        crate::db::game_servers::GameServerFields {
            name: req.name.trim(),
            mc_host: req.mc_host.trim(),
            mc_port: req.mc_port,
            sort_order: req.sort_order,
            kind: req.kind(),
            maintenance: req.maintenance,
            maintenance_reason: req.maintenance_reason.as_deref(),
        },
    )
    .await?;
    if req.maintenance {
        crate::agent_link::notify::maintenance_start(
            &state,
            Some(server_id),
            Some(id),
            req.countdown_seconds.unwrap_or(60),
            req.maintenance_reason.clone(),
        );
    } else {
        crate::agent_link::notify::maintenance_cancel(&state, Some(server_id), Some(id));
    }
    broadcast(&state);
    Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct BulkMaintenanceReq {
    pub maintenance: bool,
    pub maintenance_reason: Option<String>,
    #[serde(default)]
    pub countdown_seconds: Option<u32>,
}

/// PUT /api/admin/servers/{id}/game-servers/maintenance — включение/выключение техработ сразу для всех серверов сборки
pub async fn bulk_maintenance(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(server_id): Path<Uuid>,
    Json(req): Json<BulkMaintenanceReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_SERVERS_AGENTS)?;
    let count = crate::db::set_build_game_servers_maintenance(
        &state.db,
        server_id,
        req.maintenance,
        req.maintenance_reason.as_deref(),
    )
    .await?;
    if req.maintenance {
        crate::agent_link::notify::maintenance_start(
            &state,
            Some(server_id),
            None,
            req.countdown_seconds.unwrap_or(60),
            req.maintenance_reason.clone(),
        );
    } else {
        crate::agent_link::notify::maintenance_cancel(&state, Some(server_id), None);
    }
    broadcast(&state);
    Ok(Json(json!({ "ok": true, "updated_servers": count })))
}

pub async fn rotate_token(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((_server_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_SERVERS_AGENTS)?;
    let secret = generate_agent_secret();
    crate::db::rotate_game_server_token(&state.db, id, &hash_agent_secret(&secret)).await?;
    Ok(Json(json!({ "secret": secret })))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((_server_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_SERVERS_AGENTS)?;
    crate::db::delete_game_server(&state.db, id).await?;
    broadcast(&state);
    Ok(Json(json!({ "ok": true })))
}
