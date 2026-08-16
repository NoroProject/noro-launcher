//! Админ: запрос логов у игрока.
//!
//! Два режима. С согласием — модалка у игрока, он решает. Принудительно —
//! отдельное право, потому что в расследовании согласие бессмысленно:
//! единственный, чьи логи никогда не придут, это тот, ради кого всё затевалось.

use crate::api::auth::AdminAuth;
use crate::audit;
use crate::db::log_requests::LogRequestRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use schema::{PERM_ADMIN_SUPPORT_LOGS, PERM_ADMIN_SUPPORT_LOGS_FORCE};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RequestReq {
    pub reason: String,
    #[serde(default)]
    pub server_id: Option<Uuid>,
    /// Без спроса. Требует отдельного права.
    #[serde(default)]
    pub forced: bool,
}

pub async fn request(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(target_id): Path<Uuid>,
    Json(req): Json<RequestReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_SUPPORT_LOGS)?;
    if req.forced {
        admin.require(PERM_ADMIN_SUPPORT_LOGS_FORCE)?;
    }

    let reason = req.reason.trim();
    if reason.len() < 3 {
        return Err(AppError::BadRequest(
            "нужна причина: игрок увидит её в модалке, и она же останется в аудите".into(),
        ));
    }

    let target = crate::db::load_profile(&state.db, target_id).await?;
    let row = crate::db::create_log_request(
        &state.db,
        admin.user_id(),
        &admin.actor.label(),
        target_id,
        reason,
        req.server_id,
        req.forced,
    )
    .await?;

    state.ws.send_to_user(
        target_id,
        &schema::ServerWsMsg::LogRequest {
            request_id: row.id,
            actor_username: admin.actor.label(),
            reason: reason.to_string(),
            forced: req.forced,
            expires_at: row.expires_at,
        },
    );

    // Принудительный сбор — отдельное событие: по нему потом и спросят.
    audit::record(
        &state,
        &admin.actor,
        if req.forced {
            "support.logs.force"
        } else {
            "support.logs.request"
        },
        audit::target("user", target_id),
        json!({ "request_id": row.id, "reason": reason, "username": target.username }),
    )
    .await;

    Ok(Json(json!({
        "request_id": row.id,
        "expires_at": row.expires_at,
        "launcher_online": state.ws.is_user_connected(target_id),
    })))
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub user_id: Option<Uuid>,
    pub limit: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<LogRequestRow>>> {
    admin.require(PERM_ADMIN_SUPPORT_LOGS)?;
    // Протухшие помечаем при чтении: фонового прохода ради пяти минут заводить
    // незачем, а «ожидает ответа» на неделю — это враньё в интерфейсе.
    crate::db::expire_log_requests(&state.db).await?;
    Ok(Json(
        crate::db::list_log_requests(&state.db, q.user_id, q.limit.unwrap_or(50)).await?,
    ))
}
