//! Админ-эндпоинты управления расписанием рестартов серверов.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListRestartsQuery {
    pub game_server_id: Uuid,
}

#[derive(Deserialize)]
pub struct CreateRestartReq {
    pub game_server_id: Uuid,
    pub cron_expr: Option<String>,
    pub at_times: Option<Vec<String>>,
    pub interval_minutes: Option<i32>,
    pub notice_minutes: Option<i32>,
    pub online_policy: Option<String>,
    pub max_defer_minutes: Option<i32>,
}

/// GET /api/admin/restarts?game_server_id=...
pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<ListRestartsQuery>,
) -> AppResult<Json<Vec<crate::db::restart_schedules::RestartScheduleRow>>> {
    admin.require("noro.admin.servers.edit")?;
    Ok(Json(
        crate::db::restart_schedules::list_restart_schedules(&state.db, q.game_server_id).await?,
    ))
}

/// POST /api/admin/restarts
pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<CreateRestartReq>,
) -> AppResult<Json<crate::db::restart_schedules::RestartScheduleRow>> {
    admin.require("noro.admin.servers.edit")?;
    if req.cron_expr.is_none() && req.at_times.is_none() && req.interval_minutes.is_none() {
        return Err(AppError::BadRequest(
            "must specify cron_expr, at_times or interval_minutes".into(),
        ));
    }
    let row = crate::db::restart_schedules::create_restart_schedule(
        &state.db,
        req.game_server_id,
        req.cron_expr.as_deref(),
        req.at_times,
        req.interval_minutes,
        req.notice_minutes.unwrap_or(5),
        req.online_policy.as_deref().unwrap_or("warn_and_go"),
        req.max_defer_minutes.unwrap_or(30),
    )
    .await?;
    Ok(Json(row))
}

/// DELETE /api/admin/restarts/{id}
pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require("noro.admin.servers.edit")?;
    let ok = crate::db::restart_schedules::delete_restart_schedule(&state.db, id).await?;
    Ok(Json(json!({ "ok": ok })))
}
