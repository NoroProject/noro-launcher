//! Rebuilding the badge pack is a button, not a side effect of editing a role.
//!
//! Roles get edited in batches, and every rebuild hands every online player a
//! new pack and a resource reload. Let someone pick the moment instead.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::prefix;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use schema::PERM_ROLES_EDIT;

pub async fn sync(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<prefix::Current>> {
    admin.require(PERM_ROLES_EDIT)?;
    Ok(Json(prefix::rebuild(&state).await?))
}
