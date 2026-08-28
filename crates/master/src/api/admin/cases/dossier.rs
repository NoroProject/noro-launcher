//! Player dossier: name, roles, active punishments, confirmed case count.
//!
//! Just enough to answer "first offence or third ban for the same thing?"
//! without leaving the case mid-review. Full history stays on the player page.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::Json;
use schema::PERM_CASES_VIEW;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct DossierQuery {
    pub username: String,
}

pub async fn dossier(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(query): Query<DossierQuery>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CASES_VIEW)?;
    let user = crate::db::user_by_mc_username(&state.db, query.username.trim())
        .await?
        .ok_or_else(|| AppError::NotFound("user".into()))?;

    let roles = crate::db::load_user_roles(&state.db, user.id)
        .await
        .unwrap_or_default();
    let stats = crate::db::cases::target_stats(&state.db, user.id).await?;

    // Active only — a hover card wants the current state, not the log.
    let now = chrono::Utc::now();
    let active: Vec<_> = crate::db::list_punishments(&state.db, user.id)
        .await?
        .into_iter()
        .filter(|p| p.revoked_at.is_none() && p.expires_at.is_none_or(|e| e > now))
        .collect();

    Ok(Json(json!({
        "user_id": user.id,
        "username": user.mc_username,
        "roles": roles.into_iter().map(|r| r.name).collect::<Vec<_>>(),
        "first_seen": user.created_at,
        "cases_total": stats.total,
        "cases_confirmed": stats.confirmed,
        "active_punishments": active,
    })))
}
