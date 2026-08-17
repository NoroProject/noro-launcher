//! Админ: наказания и заметки на карточке игрока.

use super::punish_limits;
use crate::api::auth::AdminAuth;
use crate::audit;
use crate::db::punishments::{NewPunishment, PunishmentRow};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use chrono::{Duration, Utc};
use schema::{PERM_PUNISH_REVOKE, PERM_PUNISH_VIEW};

use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateReq {
    /// `ban` | `warn` | `server_ban`
    pub kind: String,
    pub reason: String,
    /// Срок в минутах. `None` — навсегда.
    ///
    /// Минуты, а не часы: панель принимает `30m` наравне с `7d`, и округлять
    /// короткий срок вверх до часа значило бы наказывать не тем, что назначили.
    #[serde(default)]
    pub minutes: Option<i64>,
    /// Только для `server_ban`.
    #[serde(default)]
    pub server_id: Option<Uuid>,
    /// Правило, по которому наказывают. Оно же задаёт рамки срока: без
    /// `noro.mod.punish.bypass` выйти за них нельзя.
    #[serde(default)]
    pub rule_id: Option<Uuid>,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<PunishmentRow>>> {
    admin.require(PERM_PUNISH_VIEW)?;
    Ok(Json(crate::db::list_punishments(&state.db, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateReq>,
) -> AppResult<Json<Value>> {
    if !matches!(req.kind.as_str(), "ban" | "warn" | "server_ban" | "mute") {
        return Err(AppError::BadRequest("unknown punishment kind".into()));
    }
    if req.reason.trim().len() < 3 {
        return Err(AppError::BadRequest(
            "a reason is required: it is the only thing that will explain this in six months"
                .into(),
        ));
    }
    if req.kind == "server_ban" && req.server_id.is_none() {
        return Err(AppError::BadRequest(
            "a server restriction needs a server".into(),
        ));
    }
    if req.kind == "ban" && crate::db::is_root_user(&state.db, id).await? {
        return Err(AppError::Forbidden("root cannot be banned".into()));
    }

    // Правило решает, что именно допустимо: вид наказания и рамки срока.
    let rule = match req.rule_id {
        Some(rule_id) => crate::db::rule_by_id(&state.db, rule_id).await?,
        None => None,
    };
    let sanctions = match &rule {
        Some(rule) => crate::db::sanctions_of_rule(&state.db, rule.id).await?,
        None => Vec::new(),
    };
    punish_limits::check(
        &punish_limits::Request {
            kind: &req.kind,
            minutes: req.minutes,
            rule_cited: rule.is_some(),
        },
        &sanctions,
        |perm| admin.require(perm).is_ok(),
    )?;

    let expires_at = req
        .minutes
        .map(|m| Utc::now() + Duration::minutes(m.max(1)));
    let row = crate::db::create_punishment(
        &state.db,
        NewPunishment {
            user_id: id,
            kind: &req.kind,
            reason: req.reason.trim(),
            actor_id: admin.user_id(),
            actor_label: &admin.actor.label(),
            server_id: req.server_id,
            expires_at,
            rule_id: rule.as_ref().map(|r| r.id),
            rule_code: rule.as_ref().map(|r| r.code.as_str()),
        },
    )
    .await?;

    // Флаг в users остаётся кэшем: по нему ходят Yggdrasil и WS-вход.
    let banned = crate::db::refresh_ban_flag(&state.db, id).await?;
    if banned {
        notify_profile(&state, id).await;
    }

    audit::record(
        &state,
        &admin.actor,
        audit::actions::punishment(&req.kind),
        audit::target("user", id),
        json!({
            "reason": row.reason,
            "expires_at": row.expires_at,
            "server_id": row.server_id,
            "rule": row.rule_code,
        }),
    )
    .await;

    Ok(Json(json!(row)))
}

pub async fn revoke(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, punishment_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_PUNISH_REVOKE)?;
    if !crate::db::revoke_punishment(&state.db, punishment_id, admin.user_id()).await? {
        return Err(AppError::NotFound(
            "punishment not found or already lifted".into(),
        ));
    }
    crate::db::refresh_ban_flag(&state.db, id).await?;
    notify_profile(&state, id).await;

    audit::record(
        &state,
        &admin.actor,
        audit::actions::PUNISHMENT_REVOKE,
        audit::target("user", id),
        json!({ "punishment_id": punishment_id }),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}

/// Разослать обновлённый профиль: игрок должен узнать о снятии бана сразу, а
/// не при следующем перезапуске лаунчера.
async fn notify_profile(state: &AppState, id: Uuid) {
    if let Ok(profile) = crate::db::load_profile(&state.db, id).await {
        state.ws.send_to_user(
            id,
            &schema::ServerWsMsg::PermissionsUpdated { user: profile },
        );
    }
}
