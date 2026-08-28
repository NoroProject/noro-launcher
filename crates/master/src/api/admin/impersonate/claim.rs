//! Trading an approved grant for a session.
//!
//! The launcher calls this with its own bearer token, so the impersonated
//! token comes back over an already-authenticated channel instead of through a
//! browser, a URL or process arguments.

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

/// Long enough to look into a problem, short enough that nobody forgets which
/// account they're in.
const SESSION_TTL_MINS: i64 = 30;

#[derive(Deserialize)]
pub struct ClaimReq {
    pub grant_id: Uuid,
}

pub async fn claim(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<ClaimReq>,
) -> AppResult<Json<Value>> {
    let grant = crate::db::consume_grant(&state.db, req.grant_id, user.user_id)
        .await?
        .ok_or_else(|| {
            AppError::Forbidden("grant not found, not approved or already used".into())
        })?;

    // Re-check: minutes can pass between requesting a grant and claiming it,
    // and a role can be taken away in that gap.
    let actor = crate::db::load_profile(&state.db, grant.actor_id).await?;
    let target = crate::db::load_profile(&state.db, grant.target_id).await?;
    if !super::can_impersonate(&actor, &target) {
        return Err(AppError::Forbidden(
            "permissions changed: the target is no longer within yours".into(),
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
        audit::actions::IMPERSONATE_START,
        audit::target("user", grant.target_id),
        json!({
            "grant_id": grant.id,
            "reason": grant.reason,
            "target_username": target.username,
        }),
    )
    .await;
    tracing::warn!(actor = %actor.username, target = %target.username, "impersonation session started");

    Ok(Json(json!({
        "access_token": access_token.to_string(),
        "user": target,
        "expires_in_secs": SESSION_TTL_MINS * 60,
    })))
}
