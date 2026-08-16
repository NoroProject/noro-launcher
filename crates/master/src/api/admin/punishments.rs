//! Админ: наказания и заметки на карточке игрока.

use crate::api::auth::AdminAuth;
use crate::audit;
use crate::db::punishments::PunishmentRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use chrono::{Duration, Utc};
use schema::{PERM_ADMIN_USERS, PERM_MOD_USERS_BAN};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateReq {
    /// `ban` | `warn` | `server_ban`
    pub kind: String,
    pub reason: String,
    /// Срок в часах. `None` — навсегда.
    #[serde(default)]
    pub hours: Option<i64>,
    /// Только для `server_ban`.
    #[serde(default)]
    pub server_id: Option<Uuid>,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<PunishmentRow>>> {
    admin.require(PERM_ADMIN_USERS)?;
    Ok(Json(crate::db::list_punishments(&state.db, id).await?))
}

pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_MOD_USERS_BAN)?;

    if !matches!(req.kind.as_str(), "ban" | "warn" | "server_ban") {
        return Err(AppError::BadRequest("неизвестный вид наказания".into()));
    }
    if req.reason.trim().len() < 3 {
        return Err(AppError::BadRequest(
            "нужна причина: она единственное, что объяснит наказание через полгода".into(),
        ));
    }
    if req.kind == "server_ban" && req.server_id.is_none() {
        return Err(AppError::BadRequest(
            "ограничение по серверу требует указать сервер".into(),
        ));
    }
    if req.kind == "ban" && crate::db::is_root_user(&state.db, id).await? {
        return Err(AppError::Forbidden("root нельзя забанить".into()));
    }

    let expires_at = req.hours.map(|h| Utc::now() + Duration::hours(h.max(1)));
    let row = crate::db::create_punishment(
        &state.db,
        id,
        &req.kind,
        req.reason.trim(),
        admin.user_id(),
        &admin.actor.label(),
        req.server_id,
        expires_at,
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
        &format!("punishment.{}", req.kind),
        audit::target("user", id),
        json!({ "reason": row.reason, "expires_at": row.expires_at, "server_id": row.server_id }),
    )
    .await;

    Ok(Json(json!(row)))
}

pub async fn revoke(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, punishment_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_MOD_USERS_BAN)?;
    if !crate::db::revoke_punishment(&state.db, punishment_id, admin.user_id()).await? {
        return Err(AppError::NotFound(
            "наказание не найдено или уже снято".into(),
        ));
    }
    crate::db::refresh_ban_flag(&state.db, id).await?;
    notify_profile(&state, id).await;

    audit::record(
        &state,
        &admin.actor,
        "punishment.revoke",
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
