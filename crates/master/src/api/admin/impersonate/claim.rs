//! Обмен подтверждённого гранта на сессию.
//!
//! Ходит сюда сам лаунчер, своим текущим Bearer: токен уезжает по уже
//! аутентифицированному каналу, а не через браузер, URL или аргументы процесса.

use crate::api::auth::AuthUser;
use crate::audit;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use chrono::Duration;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// Сколько живёт сессия impersonation. Полчаса: этого хватает разобраться в
/// проблеме, и не хватает забыть, что ты в чужом аккаунте.
const SESSION_TTL_MINS: i64 = 30;

#[derive(Deserialize)]
pub struct ClaimReq {
    pub grant_id: Uuid,
}

/// Забрать сессию по подтверждённому гранту.
pub async fn claim(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<ClaimReq>,
) -> AppResult<Json<Value>> {
    let grant = crate::db::consume_grant(&state.db, req.grant_id, user.user_id)
        .await?
        .ok_or_else(|| {
            AppError::Forbidden("грант не найден, не подтверждён или уже использован".into())
        })?;

    // Права могли измениться между запросом и подтверждением — проверяем ещё
    // раз: между двумя проверками прошла минута, и за неё роль могли снять.
    let actor = crate::db::load_profile(&state.db, grant.actor_id).await?;
    let target = crate::db::load_profile(&state.db, grant.target_id).await?;
    if !super::can_impersonate(&actor, &target) {
        return Err(AppError::Forbidden(
            "права изменились: цель больше не внутри ваших".into(),
        ));
    }

    let access_token = crate::db::create_impersonated_session(
        &state.db,
        grant.target_id,
        grant.actor_id,
        Duration::minutes(SESSION_TTL_MINS),
    )
    .await?;

    audit::record(
        &state,
        &audit::Actor::User {
            id: actor.id,
            username: actor.username.clone(),
        },
        "impersonate.start",
        audit::target("user", grant.target_id),
        json!({
            "grant_id": grant.id,
            "reason": grant.reason,
            "target_username": target.username,
        }),
    )
    .await;
    tracing::warn!(actor = %actor.username, target = %target.username, "начата сессия impersonation");

    Ok(Json(json!({
        "access_token": access_token.to_string(),
        "user": target,
        "expires_in_secs": SESSION_TTL_MINS * 60,
    })))
}
