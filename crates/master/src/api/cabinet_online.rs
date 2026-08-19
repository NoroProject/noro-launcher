//! Настройка скрытия из публичного списка онлайна.

use crate::api::auth::AuthUser;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use schema::UserProfile;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct HideFromOnlineReq {
    pub hide: bool,
}

/// PUT /api/me/hide-from-online — тумблер скрытия из публичного списка.
pub async fn set_hide_from_online(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<HideFromOnlineReq>,
) -> AppResult<Json<UserProfile>> {
    sqlx::query("UPDATE users SET hide_from_online = $1 WHERE id = $2")
        .bind(req.hide)
        .bind(user.user_id)
        .execute(&state.db)
        .await?;
    Ok(Json(
        crate::db::load_profile(&state.db, user.user_id).await?,
    ))
}

#[derive(Deserialize)]
pub struct SilentJoinReq {
    pub silent: bool,
}

/// PUT /api/me/silent-join — тумблер автованиша при входе.
pub async fn set_silent_join(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<SilentJoinReq>,
) -> AppResult<Json<UserProfile>> {
    let profile = crate::db::load_profile(&state.db, user.user_id).await?;
    if !profile.has_permission("noro.mod.vanish.silent_join")
        && !profile.has_permission("noro.mod.vanish.use")
    {
        return Err(crate::error::AppError::Forbidden(
            "no permission for silent join".into(),
        ));
    }
    sqlx::query("UPDATE users SET silent_join = $1 WHERE id = $2")
        .bind(req.silent)
        .bind(user.user_id)
        .execute(&state.db)
        .await?;
    Ok(Json(
        crate::db::load_profile(&state.db, user.user_id).await?,
    ))
}
