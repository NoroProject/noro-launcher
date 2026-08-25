//! Админ: диагностика лаунчера и удалённые действия.
//!
//! Диагностика личного не содержит — версии, железо, скорость до мастера, — и
//! согласия не требует. Действия, наоборот, идут через подтверждение у игрока:
//! всё, что стирает файлы, это уже не поддержка, а управление чужой машиной.

use crate::api::auth::AdminAuth;
use crate::audit;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use schema::{RemoteAction, PERM_USERS_LAUNCHER};

use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// Попросить лаунчер прислать диагностику.
pub async fn request_diagnostics(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_USERS_LAUNCHER)?;
    if !state.ws.is_user_connected(id) {
        return Err(AppError::state(
            crate::error_codes::LAUNCHER_OFFLINE,
            "launcher is offline",
        ));
    }
    state
        .ws
        .send_to_user(id, &schema::ServerWsMsg::RequestDiagnostics);
    Ok(Json(json!({ "ok": true })))
}

/// Последняя присланная диагностика.
pub async fn diagnostics(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_USERS_LAUNCHER)?;
    Ok(Json(
        match crate::db::latest_diagnostics(&state.db, id).await? {
            Some(v) => v,
            None => Value::Null,
        },
    ))
}

#[derive(Deserialize)]
pub struct ActionReq {
    pub action: RemoteAction,
    #[serde(default)]
    pub server_id: Option<Uuid>,
}

/// Попросить лаунчер что-то сделать.
pub async fn run_action(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<ActionReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_USERS_LAUNCHER)?;
    if !state.ws.is_user_connected(id) {
        return Err(AppError::state(
            crate::error_codes::LAUNCHER_OFFLINE,
            "launcher is offline",
        ));
    }

    state.ws.send_to_user(
        id,
        &schema::ServerWsMsg::RemoteAction {
            action: req.action,
            server_id: req.server_id,
            actor_username: admin.actor.label(),
        },
    );

    audit::record(
        &state,
        &admin.actor,
        audit::actions::REMOTE_ACTION,
        audit::target("user", id),
        json!({ "action": req.action.as_str(), "server_id": req.server_id }),
    )
    .await;

    Ok(Json(json!({
        "ok": true,
        "needs_confirmation": req.action.needs_confirmation(),
    })))
}
