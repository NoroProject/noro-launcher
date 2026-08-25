//! История наказаний, снятие и подтверждение варна — со стороны игры.

use crate::api::auth::AgentAuth;
use crate::db::punishments::PunishmentRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// Кто наказывает или снимает: живой модератор со своими правами либо сам
/// сервер.
pub struct Actor {
    pub id: Option<Uuid>,
    pub label: String,
    /// `None` — прав нет вовсе: наказывает сервер, а не человек.
    pub permissions: Option<Vec<String>>,
}

impl Actor {
    pub fn audit_actor(&self) -> crate::audit::Actor {
        match self.id {
            Some(id) => crate::audit::Actor::User {
                id,
                username: self.label.clone(),
            },
            None => crate::audit::Actor::Token {
                name: self.label.clone(),
            },
        }
    }

    pub fn require(&self, perm: &str) -> AppResult<()> {
        match &self.permissions {
            Some(permissions)
                if schema::any_permission_matches(permissions.iter().map(String::as_str), perm) =>
            {
                Ok(())
            }
            // Сервер без модератора за спиной: снимать наказания сам он не
            // может — иначе доступ к серверной консоли равнялся бы амнистии.
            _ => Err(AppError::Forbidden(format!(
                "the {perm} permission is required"
            ))),
        }
    }
}

/// Модератор по его MC UUID. Права берём на сборке этого агента — те же, что
/// панель показывает на карточке игрока.
pub async fn resolve_actor(
    state: &AppState,
    actor_uuid: Option<Uuid>,
    agent: &AgentAuth,
) -> AppResult<Actor> {
    let Some(actor_uuid) = actor_uuid else {
        return Ok(Actor {
            id: None,
            label: format!("Agent: {}", agent.game_server.name),
            permissions: None,
        });
    };
    let user = crate::db::user_by_mc_uuid(&state.db, actor_uuid)
        .await?
        .ok_or_else(|| AppError::Unauthorized("the moderator has no account here".into()))?;
    let permissions =
        crate::db::effective_permissions(&state.db, user.id, agent.game_server.server_id).await?;
    Ok(Actor {
        id: Some(user.id),
        label: user.mc_username,
        permissions: Some(permissions),
    })
}

#[derive(Deserialize)]
pub struct ActorReq {
    /// MC UUID того, кто набрал команду.
    #[serde(default)]
    pub actor_uuid: Option<Uuid>,
}

/// `GET /api/agent/players/{mc_uuid}/punishments` — вся история, включая
/// снятое: по ней модератор в игре решает, что выдавать дальше.
pub async fn list_punishments(
    State(state): State<AppState>,
    _agent: AgentAuth,
    Path(mc_uuid): Path<Uuid>,
) -> AppResult<Json<Vec<PunishmentRow>>> {
    let row = crate::db::user_by_mc_uuid(&state.db, mc_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound("player not found".into()))?;
    Ok(Json(crate::db::list_punishments(&state.db, row.id).await?))
}

/// `POST /api/agent/punishments/{id}/revoke` — `/unban`, `/unmute`.
pub async fn revoke_punishment(
    State(state): State<AppState>,
    agent: AgentAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<ActorReq>,
) -> AppResult<Json<Value>> {
    let actor = resolve_actor(&state, req.actor_uuid, &agent).await?;
    actor.require(schema::PERM_PUNISH_REVOKE)?;

    let punishment = crate::db::punishment_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("no such punishment".into()))?;
    if !crate::db::revoke_punishment(&state.db, id, actor.id).await? {
        return Err(AppError::state(
            crate::error_codes::ALREADY_LIFTED,
            "it is already lifted",
        ));
    }
    if punishment.kind == "ban" {
        crate::db::refresh_ban_flag(&state.db, punishment.user_id).await?;
    }
    crate::agent_link::notify::revoked(&state, &punishment, &actor.label).await;

    crate::audit::record(
        &state,
        &actor.audit_actor(),
        crate::audit::actions::PUNISHMENT_REVOKE,
        crate::audit::target("user", punishment.user_id),
        json!({ "kind": punishment.kind, "from": agent.game_server.name }),
    )
    .await;

    Ok(Json(json!({ "ok": true })))
}

/// `POST /api/agent/punishments/{id}/ack` — игрок прочитал предупреждение в
/// игре. Подтверждает он сам за себя, поэтому прав здесь не требуется.
pub async fn acknowledge(
    State(state): State<AppState>,
    _agent: AgentAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<ActorReq>,
) -> AppResult<Json<Value>> {
    let uuid = req
        .actor_uuid
        .ok_or_else(|| AppError::BadRequest("actor_uuid is required".into()))?;
    let user = crate::db::user_by_mc_uuid(&state.db, uuid)
        .await?
        .ok_or_else(|| AppError::NotFound("player not found".into()))?;
    let done = crate::db::acknowledge_punishment(&state.db, id, user.id).await?;
    Ok(Json(json!({ "ok": done })))
}
