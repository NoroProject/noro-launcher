//! Админ-эндпоинты управления расписанием рестартов серверов.

use crate::api::auth::AdminAuth;
use crate::api::created::created;
use crate::api::paging::Page;
use crate::api::validate::Validation;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::response::Response;
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
) -> AppResult<Json<Page<crate::db::restart_schedules::RestartScheduleRow>>> {
    admin.require("noro.admin.servers.edit")?;
    Ok(Json(Page::whole(
        crate::db::restart_schedules::list_restart_schedules(&state.db, q.game_server_id).await?,
    )))
}

/// POST /api/admin/restarts
pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<CreateRestartReq>,
) -> AppResult<Response> {
    admin.require("noro.admin.servers.edit")?;
    Validation::new()
        .any_of(
            "cron_expr",
            req.cron_expr.is_some() || req.at_times.is_some() || req.interval_minutes.is_some(),
        )
        .positive("interval_minutes", req.interval_minutes.map(i64::from))
        .positive("max_defer_minutes", req.max_defer_minutes.map(i64::from))
        .rule(
            "notice_minutes",
            req.notice_minutes.is_none_or(|m| m >= 0),
            "out_of_range",
            "must not be negative",
        )
        .one_of(
            "online_policy",
            req.online_policy.as_deref().unwrap_or("warn_and_go"),
            &["warn_and_go", "wait_for_empty", "force"],
        )
        .finish()?;
    let row = crate::db::restart_schedules::create_restart_schedule(
        &state.db,
        crate::db::restart_schedules::NewRestartSchedule {
            game_server_id: req.game_server_id,
            cron_expr: req.cron_expr.as_deref(),
            at_times: req.at_times,
            interval_minutes: req.interval_minutes,
            notice_minutes: req.notice_minutes.unwrap_or(5),
            online_policy: req.online_policy.as_deref().unwrap_or("warn_and_go"),
            max_defer_minutes: req.max_defer_minutes.unwrap_or(30),
        },
    )
    .await?;
    Ok(created(format!("/api/admin/restarts/{}", row.id), row))
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
