//! Админ: роли и их права.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::{Path, State, Query};
use axum::Json;
use schema::{Role, PERM_ADMIN_ROLES};
use serde::Deserialize;
use uuid::Uuid;

pub async fn list(State(state): State<AppState>, admin: AdminAuth) -> AppResult<Json<Vec<Role>>> {
    admin.require(PERM_ADMIN_ROLES)?;
    Ok(Json(crate::db::list_roles(&state.db).await?))
}

#[derive(Deserialize)]
pub struct CreateReq {
    pub name: String,
    pub display_name: String,
    pub color: Option<String>,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub sort_order: i32,
}

pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<CreateReq>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_ADMIN_ROLES)?;
    let id = crate::db::create_role(
        &state.db,
        &req.name,
        &req.display_name,
        req.color.as_deref(),
        req.is_default,
        req.sort_order,
    )
    .await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

#[derive(Deserialize)]
pub struct UpdateReq {
    pub display_name: String,
    pub color: Option<String>,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub sort_order: i32,
    /// Имя группы LuckPerms, с которой связана роль.
    #[serde(default)]
    pub lp_group: Option<String>,
    /// Иконка роли: имя из набора либо юникод-символ.
    #[serde(default)]
    pub icon: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateReq>,
) -> AppResult<Json<Role>> {
    admin.require(PERM_ADMIN_ROLES)?;
    crate::db::update_role(
        &state.db,
        id,
        &req.display_name,
        req.color.as_deref(),
        req.is_default,
        req.sort_order,
        req.lp_group.as_deref(),
        req.icon.as_deref(),
    )
    .await?;
    let roles = crate::db::list_roles(&state.db).await?;
    Ok(Json(roles.into_iter().find(|r| r.id == id).unwrap_or_else(
        || Role {
            id,
            name: String::new(),
            display_name: req.display_name,
            color: req.color,
            permissions: vec![],
            is_default: req.is_default,
            sort_order: req.sort_order,
            lp_group: req.lp_group,
            icon: req.icon,
        },
    )))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_ADMIN_ROLES)?;
    crate::db::delete_role(&state.db, id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
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
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_ADMIN_ROLES)?;
    crate::db::add_role_permission(&state.db, id, &req.permission, req.server_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn remove_permission(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, perm)): Path<(Uuid, String)>,
    Query(scope): Query<PermScope>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_ADMIN_ROLES)?;
    crate::db::remove_role_permission(&state.db, id, &perm, scope.server_id).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
