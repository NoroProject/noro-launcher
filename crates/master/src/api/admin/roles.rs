//! Админ: роли и их права.

use crate::api::auth::AdminAuth;
use crate::api::created::created;
use crate::api::paging::Page;
use crate::api::validate::Validation;
use crate::audit::{self, target};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::Json;
use schema::{Role, PERM_ROLES_EDIT, PERM_ROLES_VIEW};

use serde::Deserialize;
use uuid::Uuid;

pub async fn list(State(state): State<AppState>, admin: AdminAuth) -> AppResult<Json<Page<Role>>> {
    admin.require(PERM_ROLES_VIEW)?;
    Ok(Json(Page::whole(crate::db::list_roles(&state.db).await?)))
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
) -> AppResult<Response> {
    admin.require(PERM_ROLES_EDIT)?;
    // Роли без имени раньше доезжали до базы: проверки здесь не было вовсе, а
    // в списке такая роль выглядела пустой строкой, которую не за что нажать.
    Validation::new()
        .required("name", &req.name)
        .max_len("name", &req.name, 32)
        .required("display_name", &req.display_name)
        .max_len("display_name", &req.display_name, 32)
        .finish()?;

    let id = crate::db::create_role(
        &state.db,
        &req.name,
        &req.display_name,
        req.color.as_deref(),
        req.is_default,
        req.sort_order,
    )
    .await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::ROLE_CREATE,
        target("role", id),
        serde_json::json!({ "name": req.name, "display_name": req.display_name }),
    )
    .await;
    Ok(created(
        format!("/api/admin/roles/{id}"),
        serde_json::json!({ "id": id }),
    ))
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
    /// Что стоит перед ником в игре, с цветами через `&`. Иконка — не префикс:
    /// она остаётся одним глифом для таба и сайта.
    #[serde(default)]
    pub prefix: Option<String>,
    /// Что стоит после ника. Тот же формат.
    #[serde(default)]
    pub suffix: Option<String>,
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
    admin.require(PERM_ROLES_EDIT)?;
    if let Some(parent) = req.parent_id {
        reject_cycle(&state, id, parent).await?;
    }
    crate::db::update_role(
        &state.db,
        id,
        crate::db::RoleFields {
            display_name: &req.display_name,
            color: req.color.as_deref(),
            is_default: req.is_default,
            sort_order: req.sort_order,
            lp_group: req.lp_group.as_deref(),
            icon: req.icon.as_deref(),
            prefix: req.prefix.as_deref(),
            suffix: req.suffix.as_deref(),
            parent_id: req.parent_id,
        },
    )
    .await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::ROLE_UPDATE,
        target("role", id),
        serde_json::json!({
            "display_name": req.display_name,
            "parent_id": req.parent_id,
            "lp_group": req.lp_group,
        }),
    )
    .await;
    everyone_rereads(&state);
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
            prefix: req.prefix,
            suffix: req.suffix,
            // Запасной ответ на случай, когда роль не нашлась сразу после
            // правки: картинку он не знает, и врать про неё не надо.
            badge_sha1: None,
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
            "a role cannot inherit from itself".into(),
        ));
    }
    let links = crate::db::role_parent_links(&state.db).await?;
    let mut current = Some(parent);
    for _ in 0..links.len() {
        let Some(node) = current else { return Ok(()) };
        if node == role {
            return Err(AppError::BadRequest(
                "inheritance loop: this role is already further up the chain".into(),
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
    admin.require(PERM_ROLES_EDIT)?;
    crate::db::delete_role(&state.db, id).await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::ROLE_DELETE,
        target("role", id),
        serde_json::json!({}),
    )
    .await;
    everyone_rereads(&state);
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
    admin.require(PERM_ROLES_EDIT)?;
    crate::db::add_role_permission(&state.db, id, &req.permission, req.server_id).await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::ROLE_PERM_ADD,
        target("role", id),
        serde_json::json!({ "permission": req.permission, "server_id": req.server_id }),
    )
    .await;
    everyone_rereads(&state);
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn remove_permission(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((id, perm)): Path<(Uuid, String)>,
    Query(scope): Query<PermScope>,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_ROLES_EDIT)?;
    crate::db::remove_role_permission(&state.db, id, &perm, scope.server_id).await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::ROLE_PERM_REMOVE,
        target("role", id),
        serde_json::json!({ "permission": perm, "server_id": scope.server_id }),
    )
    .await;
    everyone_rereads(&state);
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Правка роли задевает каждого её носителя, а кто в игре — знает агент.
///
/// Перечислять носителей здесь значило бы повторить выборку, которую агент всё
/// равно сделает по своему списку онлайна, — и сделать её по всей базе вместо
/// десятка человек на сервере.
fn everyone_rereads(state: &AppState) {
    crate::agent_link::notify::profile_changed(state, None);
}
