//! Запросы из дела: срез чата, инвентарь цели, проверка её клиента.
//!
//! Все три асинхронны по природе — ответ приходит кадром от агента или от
//! лаунчера и падает в ленту. Ручка отвечает «попросили», а не результатом:
//! держать HTTP-запрос открытым, пока игрок откроет лаунчер, невозможно.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use schema::{PERM_CASES_CHAT, PERM_CASES_CLIENT, PERM_CASES_INVENTORY};
use serde_json::json;
use uuid::Uuid;

/// Сколько отматывать назад по буферу агента. Десять минут — столько живёт
/// повод: жалоба пишется по горячим следам, а не через полчаса.
const CHAT_WINDOW_SECS: u32 = 600;

/// POST /api/admin/cases/{id}/chat-request
pub async fn request_chat(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CASES_CHAT)?;
    let (case, target) = case_and_target(&state, id).await?;
    crate::agent_link::cases::request_chat(&state, &case, target.mc_uuid, CHAT_WINDOW_SECS);
    Ok(Json(json!({ "ok": true })))
}

/// POST /api/admin/cases/{id}/inventory-request
pub async fn request_inventory(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CASES_INVENTORY)?;
    let (case, target) = case_and_target(&state, id).await?;
    crate::agent_link::cases::request_inventory(&state, &case, target.mc_uuid);
    Ok(Json(json!({ "ok": true })))
}

/// POST /api/admin/cases/{id}/client-check
///
/// Просит лаунчер цели проверить целостность сборки. Игрок не в лаунчере —
/// честный отказ: молча «попросили» на офлайн-клиента вводит в заблуждение.
pub async fn request_client_check(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CASES_CLIENT)?;
    let (_, target) = case_and_target(&state, id).await?;

    if !state.ws.is_user_connected(target.id) {
        return Err(AppError::state(
            crate::error_codes::LAUNCHER_OFFLINE,
            "launcher is offline",
        ));
    }
    let actor_name = admin.actor.label();
    state.ws.send_to_user(
        target.id,
        &schema::ServerWsMsg::RemoteAction {
            action: schema::RemoteAction::VerifyIntegrity,
            server_id: None,
            actor_username: actor_name.clone(),
        },
    );

    crate::cases::event(
        &state,
        id,
        admin.actor.id(),
        &actor_name,
        "web",
        "client_check",
        json!({ "target": target.mc_username }),
    )
    .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn case_and_target(
    state: &AppState,
    id: Uuid,
) -> AppResult<(crate::db::cases::CaseRow, crate::db::models::UserRow)> {
    let case = crate::db::get_case(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("case".into()))?;
    let target = crate::db::get_user(&state.db, case.target_id)
        .await?
        .ok_or_else(|| AppError::NotFound("target".into()))?;
    Ok((case, target))
}
