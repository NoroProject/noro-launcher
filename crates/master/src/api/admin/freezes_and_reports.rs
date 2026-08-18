//! Управление заморозкой игроков и репортами в админке.

use crate::api::auth::AdminAuth;
use crate::audit;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use schema::{PERM_FREEZE, PERM_REPORTS_RESOLVE, PERM_REPORTS_VIEW};

#[derive(Deserialize)]
pub struct FreezeReq {
    pub target: Uuid,
    pub reason: String,
}

#[derive(Deserialize)]
pub struct ReportsQuery {
    #[serde(default)]
    pub open_only: bool,
}

#[derive(Deserialize)]
pub struct ResolveReportReq {
    pub resolution: String,
    pub punishment_id: Option<Uuid>,
}

/// POST /api/admin/freezes
pub async fn freeze(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<FreezeReq>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_FREEZE)?;
    if req.reason.trim().is_empty() {
        return Err(AppError::BadRequest("reason cannot be empty".into()));
    }
    let user = match crate::db::get_user(&state.db, req.target).await? {
        Some(u) => u,
        None => crate::db::user_by_mc_uuid(&state.db, req.target)
            .await?
            .ok_or_else(|| AppError::NotFound("user".into()))?,
    };

    crate::db::freeze_player(&state.db, user.id, admin.actor.id(), &req.reason).await?;
    crate::agent_link::notify::profile_changed(&state, Some(user.mc_uuid));

    audit::record(
        &state,
        &admin.actor,
        audit::actions::USER_FREEZE,
        None,
        json!({ "target": user.mc_username, "reason": req.reason }),
    )
    .await;

    Ok(Json(json!({ "ok": true })))
}

/// DELETE /api/admin/freezes/{user_id}
pub async fn unfreeze(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_FREEZE)?;
    let user = match crate::db::get_user(&state.db, user_id).await? {
        Some(u) => u,
        None => crate::db::user_by_mc_uuid(&state.db, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user".into()))?,
    };

    let unfrozen = crate::db::unfreeze_player(&state.db, user.id).await?;
    if unfrozen {
        crate::agent_link::notify::profile_changed(&state, Some(user.mc_uuid));
        audit::record(
            &state,
            &admin.actor,
            audit::actions::USER_UNFREEZE,
            None,
            json!({ "target": user.mc_username }),
        )
        .await;
    }

    Ok(Json(json!({ "ok": true })))
}

/// GET /api/admin/reports
pub async fn list_reports(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(query): Query<ReportsQuery>,
) -> AppResult<Json<Vec<crate::db::reports::ReportRow>>> {
    admin.require(PERM_REPORTS_VIEW)?;
    Ok(Json(crate::db::list_reports(&state.db, query.open_only).await?))
}

/// POST /api/admin/reports/{id}/claim
pub async fn claim_report(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_REPORTS_VIEW)?;
    let actor_id = admin.actor.id().ok_or_else(|| AppError::Unauthorized("actor required".into()))?;
    let ok = crate::db::claim_report(&state.db, id, actor_id).await?;
    Ok(Json(json!({ "ok": ok })))
}

/// PUT /api/admin/reports/{id}/resolve
pub async fn resolve_report(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<ResolveReportReq>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_REPORTS_RESOLVE)?;
    let resolver_id = admin.actor.id().unwrap_or(id);
    let ok = crate::db::resolve_report(&state.db, id, resolver_id, &req.resolution, req.punishment_id).await?;
    if ok {
        audit::record(
            &state,
            &admin.actor,
            audit::actions::REPORT_RESOLVE,
            None,
            json!({ "report_id": id, "resolution": req.resolution }),
        )
        .await;
    }
    Ok(Json(json!({ "ok": ok })))
}
