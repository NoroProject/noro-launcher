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
    /// Место жалобы. Либо все четыре поля, либо ни одного: половина точки —
    /// это точка не там, а не «примерно там».
    #[serde(default)]
    pub world: Option<String>,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default)]
    pub z: Option<f64>,
}

impl CreateReportReq {
    /// Позиция целиком или её отсутствие. Всё промежуточное — сломанный агент,
    /// и молча дописать нули значит отправить модератора не туда.
    fn position(&self) -> Result<Option<(&str, f64, f64, f64)>, AppError> {
        match (self.world.as_deref(), self.x, self.y, self.z) {
            (Some(world), Some(x), Some(y), Some(z)) => Ok(Some((world, x, y, z))),
            (None, None, None, None) => Ok(None),
            _ => Err(AppError::BadRequest(
                "report position must carry world, x, y and z together".into(),
            )),
        }
    }
}

/// POST /api/agent/reports
pub async fn create_report(
    State(state): State<AppState>,
    agent: AgentAuth,
    Json(req): Json<CreateReportReq>,
) -> AppResult<Json<serde_json::Value>> {
    let at = req.position()?;
    let reporter_user = crate::db::user_by_mc_uuid(&state.db, req.reporter)
        .await?
        .ok_or_else(|| AppError::NotFound("reporter player".into()))?;
    let target_user = crate::db::user_by_mc_uuid(&state.db, req.target)
        .await?
        .ok_or_else(|| AppError::NotFound("target player".into()))?;

    if at.is_none() {
        // Не отказ: жалоба без места лучше, чем несделанная. Но в журнале это
        // видно — агент, который перестал слать позицию, иначе не всплывёт.
        tracing::warn!(
            server = %agent.game_server.id,
            reporter = %req.reporter,
            "жалоба без места: агент не прислал позицию"
        );
    }

    let report_id = crate::db::create_report(
        &state.db,
        crate::db::reports::NewReport {
            reporter_id: reporter_user.id,
            target_id: target_user.id,
            game_server_id: agent.game_server.id,
            reason: &req.reason,
            world: at.map(|p| p.0),
            x: at.map(|p| p.1),
            y: at.map(|p| p.2),
            z: at.map(|p| p.3),
        },
    )
    .await?;

    // Жалоба сразу ложится в дело: открытое на этого игрока, иначе новое.
    // Семь жалоб на одного читера должны стать одним разбором, а не семью.
    let case = crate::db::open_case(&state.db, target_user.id, Some(agent.game_server.id)).await?;
    crate::db::attach_report(&state.db, report_id, case.id).await?;
    crate::cases::event(
        &state,
        case.id,
        Some(reporter_user.id),
        &reporter_user.mc_username,
        "game",
        "report_added",
        json!({ "report_id": report_id, "reason": req.reason }),
    )
    .await?;
    // Срез чата снимается по горячим следам: через десять минут буфер агента
    // уже прокрутится, и восстановить, что там писали, будет неоткуда.
    crate::agent_link::cases::request_chat(&state, &case, target_user.mc_uuid, 600);

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

#[cfg(test)]
mod tests {
    use super::CreateReportReq;

    fn req(world: Option<&str>, x: Option<f64>, y: Option<f64>, z: Option<f64>) -> CreateReportReq {
        CreateReportReq {
            reporter: uuid::Uuid::nil(),
            target: uuid::Uuid::nil(),
            reason: "гриф".into(),
            world: world.map(str::to_string),
            x,
            y,
            z,
        }
    }

    /// Место есть целиком — оно и едет в дело.
    #[test]
    fn full_position_passes_through() {
        let full = req(Some("minecraft:overworld"), Some(1.0), Some(2.0), Some(3.0));
        let at = full.position().expect("целая позиция принимается");
        assert_eq!(at, Some(("minecraft:overworld", 1.0, 2.0, 3.0)));
    }

    /// Места нет — и его правда нет. Нули сюда не дописываются: «0 0 0» это
    /// точка в мире, и модератор поехал бы именно туда.
    #[test]
    fn missing_position_stays_missing() {
        assert_eq!(req(None, None, None, None).position().unwrap(), None);
    }

    /// Половина точки — сломанный агент. Отказ, а не догадка.
    #[test]
    fn half_a_position_is_refused() {
        assert!(req(Some("minecraft:overworld"), None, None, None)
            .position()
            .is_err());
        assert!(req(None, Some(1.0), Some(2.0), Some(3.0))
            .position()
            .is_err());
        assert!(req(Some("minecraft:overworld"), Some(1.0), None, Some(3.0))
            .position()
            .is_err());
    }
}
