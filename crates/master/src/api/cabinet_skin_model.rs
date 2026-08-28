//! Switching the model of an already uploaded skin.
//!
//! Separate from upload on purpose: only the arm geometry changes, and a player
//! may not have the original file to re-upload — the skin could have been
//! pulled in by nickname from another account.

use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use schema::UserProfile;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ModelReq {
    /// `slim` or `classic`. A string rather than a bool, to match how Yggdrasil
    /// names the model further down the chain.
    pub model: String,
}

/// `PUT /api/me/skin/model`
pub async fn set_model(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<ModelReq>,
) -> AppResult<Json<UserProfile>> {
    let slim = super::skin_model::parse_choice(&req.model)
        .ok_or_else(|| AppError::BadRequest("model: `slim` or `classic`".into()))?;

    let current = crate::db::get_user(&state.db, user.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;
    // No skin, nothing to switch. Storing the flag anyway would show the
    // chosen model on a default Steve the player never uploaded.
    let Some(skin_url) = current.skin_url.as_deref() else {
        return Err(AppError::BadRequest("upload a skin first".into()));
    };

    crate::db::set_skin(&state.db, user.user_id, Some(skin_url), slim).await?;

    let profile = crate::db::load_profile(&state.db, user.user_id).await?;
    // Same push as an upload: the launcher and the site can both be open, and
    // the other one has to redraw itself.
    state.ws.send_to_user(
        user.user_id,
        &schema::ServerWsMsg::PermissionsUpdated {
            user: profile.clone(),
        },
    );
    Ok(Json(profile))
}
