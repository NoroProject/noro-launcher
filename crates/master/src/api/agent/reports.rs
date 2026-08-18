//! Приём жалоб (репортов) от агента и забор обратной связи при входе.

use crate::api::auth::AgentAuth;
use crate::audit;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateReportReq {
    pub reporter: Uuid,
    pub target: Uuid,
    pub reason: String,
    pub world: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
}

/// POST /api/agent/reports
pub async fn create_report(
    State(state): State<AppState>,
    agent: AgentAuth,
    Json(req): Json<CreateReportReq>,
) -> AppResult<Json<serde_json::Value>> {
    let reporter_user = crate::db::user_by_mc_uuid(&state.db, req.reporter)
        .await?
        .ok_or_else(|| AppError::NotFound("reporter player".into()))?;
    let target_user = crate::db::user_by_mc_uuid(&state.db, req.target)
        .await?
        .ok_or_else(|| AppError::NotFound("target player".into()))?;

    let report_id = crate::db::create_report(
        &state.db,
        reporter_user.id,
        target_user.id,
        agent.game_server.id,
        &req.reason,
        req.world.as_deref(),
        req.x,
        req.y,
        req.z,
    )
    .await?;

    let actor = audit::Actor::User {
        id: reporter_user.id,
        username: reporter_user.mc_username,
    };
    audit::record(
        &state,
        &actor,
        audit::actions::REPORT_CREATE,
        None,
        json!({
            "report_id": report_id,
            "target": target_user.mc_username,
            "reason": req.reason,
            "server": agent.game_server.name,
            "world": req.world,
            "x": req.x,
            "y": req.y,
            "z": req.z,
        }),
    )
    .await;

    Ok(Json(json!({ "id": report_id })))
}

/// GET /api/agent/players/{mc_uuid}/report-feedbacks
pub async fn pop_report_feedbacks(
    State(state): State<AppState>,
    _agent: AgentAuth,
    Path(mc_uuid): Path<Uuid>,
) -> AppResult<Json<Vec<crate::db::reports::ReportFeedback>>> {
    let user = crate::db::user_by_mc_uuid(&state.db, mc_uuid)
        .await?
        .ok_or_else(|| AppError::NotFound("player".into()))?;
    let feedbacks = crate::db::pop_pending_report_feedbacks(&state.db, user.id).await?;
    Ok(Json(feedbacks))
}
