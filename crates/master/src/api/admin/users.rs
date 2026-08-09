//! Админ: управление пользователями.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use schema::{UserProfile, PERM_ADMIN_USERS, PERM_MOD_USERS_BAN};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<UserProfile>>> {
    admin.require(PERM_ADMIN_USERS)?;
    let rows = crate::db::list_users(
        &state.db,
        q.limit.unwrap_or(100).min(500),
        q.offset.unwrap_or(0),
    )
    .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(crate::db::profile_from_row(&state.db, r).await?);
    }
    Ok(Json(out))
}

pub async fn get(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_ADMIN_USERS)?;
    let row = crate::db::get_user(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("пользователь".into()))?;
    Ok(Json(crate::db::profile_from_row(&state.db, row).await?))
}

#[derive(Deserialize)]
pub struct BanReq {
    pub banned: bool,
    pub reason: Option<String>,
}

pub async fn ban(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<BanReq>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_MOD_USERS_BAN)?;
    crate::db::set_user_ban(&state.db, id, req.banned, req.reason.as_deref()).await?;
    let profile = crate::db::load_profile(&state.db, id).await?;
    state.ws.send_to_user(
        id,
        &schema::ServerWsMsg::PermissionsUpdated {
            user: profile.clone(),
        },
    );
    Ok(Json(profile))
}

pub async fn add_role(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, role_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_ADMIN_USERS)?;
    crate::db::add_user_role(&state.db, id, role_id, admin.user_id).await?;
    notify_user(&state, id).await
}

pub async fn remove_role(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, role_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_ADMIN_USERS)?;
    crate::db::remove_user_role(&state.db, id, role_id).await?;
    notify_user(&state, id).await
}
#[derive(Deserialize)]
pub struct PermReq {
    pub permission: String,
    /// Контекст: `None` — везде, иначе только на этой сборке.
    #[serde(default)]
    pub server_id: Option<Uuid>,
}

/// Тот же контекст, но в query — у DELETE тела нет.
#[derive(Deserialize)]
pub struct PermScope {
    #[serde(default)]
    pub server_id: Option<Uuid>,
}
pub async fn add_permission(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<PermReq>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_ADMIN_USERS)?;
    crate::db::add_user_permission(&state.db, id, &req.permission, req.server_id, admin.user_id)
        .await?;
    notify_user(&state, id).await
}

pub async fn remove_permission(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, perm)): Path<(Uuid, String)>,
    Query(scope): Query<PermScope>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_ADMIN_USERS)?;
    crate::db::remove_user_permission(&state.db, id, &perm, scope.server_id).await?;
    notify_user(&state, id).await
}
#[derive(Deserialize)]
pub struct CapeReq {
    pub cape_id: Option<Uuid>,
}

pub async fn set_cape(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<CapeReq>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_ADMIN_USERS)?;
    let cape_url = match req.cape_id {
        Some(cape_id) => Some(
            crate::db::get_cape_url(&state.db, cape_id)
                .await?
                .ok_or_else(|| AppError::NotFound("cape".into()))?,
        ),
        None => None,
    };
    crate::db::set_user_cape(&state.db, id, cape_url.as_deref()).await?;
    notify_user(&state, id).await
}

pub async fn get_capes(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<schema::UserCapesData>> {
    admin.require(PERM_ADMIN_USERS)?;
    let granted_cape_ids = crate::db::list_user_granted_cape_ids(&state.db, id).await?;
    Ok(Json(schema::UserCapesData { granted_cape_ids }))
}

pub async fn set_granted_capes(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<schema::SetUserCapesReq>,
) -> AppResult<Json<UserProfile>> {
    admin.require(PERM_ADMIN_USERS)?;
    crate::db::set_user_granted_capes(&state.db, id, &req.granted_cape_ids).await?;
    let active_cape_url = match req.active_cape_id {
        Some(cape_id) => crate::db::get_cape_url(&state.db, cape_id).await?,
        None => None,
    };
    crate::db::set_user_cape(&state.db, id, active_cape_url.as_deref()).await?;
    notify_user(&state, id).await
}

async fn notify_user(state: &AppState, id: Uuid) -> AppResult<Json<UserProfile>> {
    let profile = crate::db::load_profile(&state.db, id).await?;
    state.ws.send_to_user(
        id,
        &schema::ServerWsMsg::PermissionsUpdated {
            user: profile.clone(),
        },
    );
    Ok(Json(profile))
}
