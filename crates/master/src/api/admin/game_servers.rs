//! Админ: игровые сервера сборки и их секреты.

use crate::api::auth::{generate_agent_secret, hash_agent_secret, AdminAuth};
use crate::db::game_servers::GameServerRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use schema::PERM_ADMIN_SERVERS;
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
) -> AppResult<Json<Vec<GameServerResp>>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    let rows = crate::db::list_game_servers(&state.db, server_id).await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| GameServerResp {
                live: row.live(),
                row,
            })
            .collect(),
    ))
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
    /// `proxy` — точка входа, `server` — бэкенд. Всё, что не `proxy`,
    /// считается бэкендом: неизвестный тип не должен выключать подсчёт онлайна.
    #[serde(default)]
    pub kind: Option<String>,
}

impl SaveReq {
    fn kind(&self) -> &str {
        match self.kind.as_deref() {
            Some("proxy") => "proxy",
            _ => "server",
        }
    }
}

/// Создать сервер. Секрет возвращается ОДИН раз: в базе только его хеш.
pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(server_id): Path<Uuid>,
    Json(req): Json<SaveReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    if crate::db::get_server(&state.db, server_id).await?.is_none() {
        return Err(AppError::NotFound("сборка не найдена".into()));
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
    Path((_server_id, id)): Path<(Uuid, Uuid)>,
    Json(req): Json<SaveReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    crate::db::update_game_server(
        &state.db,
        id,
        req.name.trim(),
        req.mc_host.trim(),
        req.mc_port,
        req.sort_order,
        req.kind(),
    )
    .await?;
    broadcast(&state);
    Ok(Json(json!({ "ok": true })))
}

/// Перевыпустить секрет: старый перестаёт работать сразу. Нужно и при утечке,
/// и когда секрет просто потеряли — показать прежний мастер уже не может.
pub async fn rotate_token(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((_server_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    let secret = generate_agent_secret();
    crate::db::rotate_game_server_token(&state.db, id, &hash_agent_secret(&secret)).await?;
    Ok(Json(json!({ "secret": secret })))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((_server_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_SERVERS)?;
    crate::db::delete_game_server(&state.db, id).await?;
    broadcast(&state);
    Ok(Json(json!({ "ok": true })))
}
