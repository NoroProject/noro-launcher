//! Личный кабинет: профиль, смена ника, скин.

use crate::api::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, State};
use axum::Json;
use schema::UserProfile;
use serde::Deserialize;

pub async fn me(user: AuthUser) -> Json<UserProfile> {
    Json(user.profile)
}

#[derive(Deserialize)]
pub struct UsernameReq {
    pub username: String,
}

fn valid_username(name: &str) -> bool {
    (3..=16).contains(&name.len()) && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub async fn set_username(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<UsernameReq>,
) -> AppResult<Json<UserProfile>> {
    if !valid_username(&req.username) {
        return Err(AppError::BadRequest(
            "username: 3-16 chars, latin letters, digits, or underscore".into(),
        ));
    }
    let taken: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE mc_username = $1 AND id <> $2)",
    )
    .bind(&req.username)
    .bind(user.user_id)
    .fetch_one(&state.db)
    .await?;
    if taken {
        return Err(AppError::Conflict("username is taken".into()));
    }
    crate::db::set_username(&state.db, user.user_id, &req.username).await?;
    Ok(Json(
        crate::db::load_profile(&state.db, user.user_id).await?,
    ))
}

/// Загрузка скина (multipart, поле `skin`, PNG).
pub async fn upload_skin(
    State(state): State<AppState>,
    user: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<UserProfile>> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if field.name() == Some("skin") {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?;
            // Простая проверка PNG-сигнатуры.
            if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
                return Err(AppError::BadRequest("PNG expected".into()));
            }
            if data.len() > 256 * 1024 {
                return Err(AppError::BadRequest("skin is too large".into()));
            }
            let stored = state
                .files
                .put_bytes(&data)
                .await
                .map_err(AppError::Other)?;
            let url = state.config.file_url(&stored.sha1);
            crate::db::set_skin(&state.db, user.user_id, Some(&url)).await?;
            return Ok(Json(
                crate::db::load_profile(&state.db, user.user_id).await?,
            ));
        }
    }
    Err(AppError::BadRequest("missing skin field".into()))
}

pub async fn delete_skin(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<UserProfile>> {
    crate::db::set_skin(&state.db, user.user_id, None).await?;
    Ok(Json(
        crate::db::load_profile(&state.db, user.user_id).await?,
    ))
}

pub async fn list_capes(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<schema::CapeRow>>> {
    let is_admin = user.profile.has_permission(schema::PERM_ADMIN_USERS);
    if is_admin {
        Ok(Json(crate::db::list_capes(&state.db).await?))
    } else {
        Ok(Json(crate::db::list_capes_for_user(&state.db, user.user_id).await?))
    }
}

pub async fn set_cape(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<schema::SelectCapeReq>,
) -> AppResult<Json<UserProfile>> {
    let is_admin = user.profile.has_permission(schema::PERM_ADMIN_USERS);
    let cape_url = match req.cape_id {
        Some(cape_id) => {
            if !is_admin {
                let allowed_ids = crate::db::list_user_granted_cape_ids(&state.db, user.user_id).await?;
                if !allowed_ids.contains(&cape_id) {
                    return Err(AppError::Forbidden("access to this cape is not granted".into()));
                }
            }
            Some(
                crate::db::get_cape_url(&state.db, cape_id)
                    .await?
                    .ok_or_else(|| AppError::NotFound("cape".into()))?,
            )
        }
        None => None,
    };
    crate::db::set_user_cape(&state.db, user.user_id, cape_url.as_deref()).await?;
    Ok(Json(
        crate::db::load_profile(&state.db, user.user_id).await?,
    ))
}
