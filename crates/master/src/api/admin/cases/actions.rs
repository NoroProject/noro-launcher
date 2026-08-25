//! Взять дело, отпустить, закрыть с вердиктом, приписать заметку.

use crate::api::auth::AdminAuth;
use crate::audit;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use schema::{PERM_CASES_CLAIM, PERM_CASES_RESOLVE, PERM_CASES_VIEW};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ResolveReq {
    /// `confirmed`, `rejected` или `insufficient`.
    pub verdict: String,
    #[serde(default)]
    pub resolution: String,
    #[serde(default)]
    pub rule_code: Option<String>,
}

#[derive(Deserialize)]
pub struct NoteReq {
    pub text: String,
}

/// POST /api/admin/cases/{id}/claim
pub async fn claim(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CASES_CLAIM)?;
    let actor_id = admin
        .actor
        .id()
        .ok_or_else(|| AppError::Unauthorized("actor required".into()))?;

    let taken = crate::db::claim_case(&state.db, id, actor_id).await?;
    if !taken {
        // Проигранная гонка: дело уже у кого-то. Это конфликт состояния, а не
        // успех — раньше отдавалось 200 с `{"ok": false}`, и клиенту, чтобы
        // понять, что действие не прошло, приходилось читать тело.
        return Err(AppError::coded(
            StatusCode::CONFLICT,
            crate::error_codes::ALREADY_CLAIMED,
            "another moderator has taken this case",
        ));
    }

    let actor_name = admin.actor.label();
    crate::cases::event(
        &state,
        id,
        Some(actor_id),
        &actor_name,
        "web",
        "claimed",
        json!({}),
    )
    .await?;

    // Модератор в игре получает режим разбора. Если он не в игре — кадр просто
    // никого не найдёт, и это нормальный случай, а не сбой.
    if let (Some(case), Ok(Some(user))) = (
        crate::db::get_case(&state.db, id).await?,
        crate::db::get_user(&state.db, actor_id).await,
    ) {
        crate::agent_link::cases::assigned(&state, &case, user.mc_uuid).await;
    }

    Ok(Json(json!({ "ok": true })))
}

/// POST /api/admin/cases/{id}/release
pub async fn release(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CASES_CLAIM)?;
    let actor_id = admin
        .actor
        .id()
        .ok_or_else(|| AppError::Unauthorized("actor required".into()))?;

    let released = crate::db::release_case(&state.db, id, actor_id).await?;
    if released {
        let actor_name = admin.actor.label();
        crate::cases::event(
            &state,
            id,
            Some(actor_id),
            &actor_name,
            "web",
            "released",
            json!({}),
        )
        .await?;
        finish_in_game(&state, id, actor_id, false).await;
    }
    Ok(Json(json!({ "ok": released })))
}

/// PUT /api/admin/cases/{id}/resolve
pub async fn resolve(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<ResolveReq>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CASES_RESOLVE)?;
    if !matches!(
        req.verdict.as_str(),
        "confirmed" | "rejected" | "insufficient"
    ) {
        return Err(AppError::BadRequest("unknown verdict".into()));
    }

    let closed = crate::db::resolve_case(
        &state.db,
        id,
        &req.verdict,
        &req.resolution,
        req.rule_code.as_deref(),
    )
    .await?;
    if !closed {
        // Дело уже закрыто — второй вердикт по нему не ставится.
        return Err(AppError::coded(
            StatusCode::CONFLICT,
            crate::error_codes::CASE_NOT_OPEN,
            "this case is already resolved",
        ));
    }

    // Жалобы закрываются вместе с делом, их авторы получают ответ при входе.
    let reports = crate::db::close_reports(&state.db, id, &req.resolution).await?;

    let actor_id = admin.actor.id();
    let actor_name = admin.actor.label();
    crate::cases::event(
        &state,
        id,
        actor_id,
        &actor_name,
        "web",
        "verdict",
        json!({ "verdict": req.verdict, "resolution": req.resolution, "rule_code": req.rule_code }),
    )
    .await?;

    if let Some(actor_id) = actor_id {
        finish_in_game(&state, id, actor_id, true).await;
    }

    audit::record(
        &state,
        &admin.actor,
        audit::actions::REPORT_RESOLVE,
        None,
        json!({ "case_id": id, "verdict": req.verdict, "reports": reports }),
    )
    .await;

    Ok(Json(json!({ "ok": true, "reports": reports })))
}

/// POST /api/admin/cases/{id}/notes
pub async fn note(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<NoteReq>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CASES_VIEW)?;
    if req.text.trim().is_empty() {
        return Err(AppError::BadRequest("note cannot be empty".into()));
    }
    let actor_name = admin.actor.label();
    crate::cases::event(
        &state,
        id,
        admin.actor.id(),
        &actor_name,
        "web",
        "note",
        json!({ "text": req.text }),
    )
    .await?;
    Ok(Json(json!({ "ok": true })))
}

/// Вывести модератора из режима разбора в игре.
async fn finish_in_game(state: &AppState, case_id: Uuid, actor_id: Uuid, closed: bool) {
    let (Ok(Some(case)), Ok(Some(user))) = (
        crate::db::get_case(&state.db, case_id).await,
        crate::db::get_user(&state.db, actor_id).await,
    ) else {
        return;
    };
    crate::agent_link::cases::finished(state, &case, user.mc_uuid, closed);
}
