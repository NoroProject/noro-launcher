//! Цитата: модератор ткнул в строку чата, которую видел своими глазами.
//!
//! Клиент не источник доказательств — он лишь указывает на них. Мод шлёт
//! отправителя, время и хеш текста; в дело едет строка из среза, снятого
//! агентом. Подделать переписку правым кликом поэтому невозможно.
//!
//! Строки в срезе ещё нет — просим срез у сервера тем же путём, что и кнопка
//! «спросить игру»: она приедет и встанет в дело сама.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use chrono::{DateTime, Utc};
use schema::PERM_CASES_CHAT;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

/// Насколько расходятся часы клиента и сервера, чтобы строку всё ещё считать
/// той же. Пять секунд: больше — и в окно попадёт соседняя реплика.
const MATCH_WINDOW_SECS: i64 = 5;

#[derive(Deserialize)]
pub struct QuoteReq {
    pub sender: String,
    pub at: DateTime<Utc>,
    /// Хеш текста у клиента. Не доказательство, а сверка: сошёлся — мод и
    /// сервер точно про одну строку.
    #[serde(default)]
    pub hash: String,
}

/// POST /api/admin/cases/{id}/quote
pub async fn quote(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<QuoteReq>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CASES_CHAT)?;
    let case = crate::db::get_case(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("case".into()))?;

    let found =
        crate::db::cases::message_near(&state.db, id, &req.sender, req.at, MATCH_WINDOW_SECS)
            .await?;

    // Строки нет — попросим срез. Событие всё равно пишем: указание модератора
    // само по себе часть разбора, даже если доказательство подъедет позже.
    if found.is_none() {
        if let Ok(Some(target)) = crate::db::get_user(&state.db, case.target_id).await {
            crate::agent_link::cases::request_chat(&state, &case, target.mc_uuid, 600);
        }
    }

    crate::cases::event(
        &state,
        id,
        admin.actor.id(),
        &admin.actor.label(),
        "game",
        "quote",
        json!({
            "sender": req.sender,
            "at": req.at,
            "hash": req.hash,
            "content": found.as_ref().map(|m| m.content.clone()),
            "matched": found.is_some(),
        }),
    )
    .await?;

    Ok(Json(json!({ "ok": true, "matched": found.is_some() })))
}
