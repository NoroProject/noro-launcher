//! Админ: роли и их права.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
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
    /// Роль-родитель, чьи права действуют и здесь. `None` — наследования нет.
    #[serde(default)]
    pub parent_id: Option<Uuid>,
}

pub async fn update(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateReq>,
) -> AppResult<Json<Role>> {
    admin.require(PERM_ADMIN_ROLES)?;
    if let Some(parent) = req.parent_id {
        reject_cycle(&state, id, parent).await?;
    }
    crate::db::update_role(
        &state.db,
        id,
        &req.display_name,
        req.color.as_deref(),
        req.is_default,
        req.sort_order,
        req.lp_group.as_deref(),
        req.icon.as_deref(),
        req.parent_id,
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
            permission_grants: vec![],
            parent_id: req.parent_id,
            inherited_permissions: vec![],
        },
    )))
}

/// Не даёт замкнуть наследование в кольцо.
///
/// Роль не может быть родителем самой себе ни напрямую, ни через цепочку:
/// иначе «права предков» перестают быть конечным списком, а админ получает
/// иерархию, в которой ни одна роль не главнее другой. Обход ограничен числом
/// ролей — если кольцо уже есть в данных, подниматься по нему можно вечно.
async fn reject_cycle(state: &AppState, role: Uuid, parent: Uuid) -> AppResult<()> {
    if role == parent {
        return Err(AppError::BadRequest(
            "роль не может наследовать саму себя".into(),
        ));
    }
    let links = crate::db::role_parent_links(&state.db).await?;
    let mut current = Some(parent);
    for _ in 0..links.len() {
        let Some(node) = current else { return Ok(()) };
        if node == role {
            return Err(AppError::BadRequest(
                "цикл наследования: эта роль уже выше по цепочке".into(),
            ));
        }
        current = links
            .iter()
            .find(|(id, _)| *id == node)
            .and_then(|(_, p)| *p);
    }
    Ok(())
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
