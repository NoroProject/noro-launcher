//! Админ: API-токены для CLI/CI.

use crate::api::auth::{admin_token, AdminAuth};
use crate::api::created::created;
use crate::api::paging::Page;
use crate::api::validate::Validation;
use crate::audit::{self, target};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::response::Response;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

/// Токен в списке. Ни селектора, ни хеша: прежний список отдавал в браузер
/// `token_hash`, который тогда был и рабочим доказательством владения.
#[derive(Serialize)]
pub struct TokenItem {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    /// Токен ещё на старой схеме хеширования — перейдёт при первом
    /// использовании. Видно в админке, чтобы забытые токены можно было отозвать.
    pub legacy_hash: bool,
}

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Page<TokenItem>>> {
    admin.require(schema::PERM_ADMIN_ALL)?;
    let items = crate::db::list_admin_tokens(&state.db)
        .await?
        .into_iter()
        .map(|t| TokenItem {
            id: t.id,
            name: t.name,
            permissions: t.permissions,
            created_at: t.created_at,
            last_used_at: t.last_used_at,
            legacy_hash: t.token_hash.is_none(),
        })
        .collect();
    Ok(Json(Page::whole(items)))
}

#[derive(Deserialize)]
pub struct CreateReq {
    pub name: String,
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// Создать токен. Секрет показывается ОДИН раз в ответе.
pub async fn create(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<CreateReq>,
) -> AppResult<Response> {
    admin.require(schema::PERM_ADMIN_ALL)?;
    // Имя — единственное, по чему токен потом опознают в списке и в аудите.
    Validation::new()
        .required("name", &req.name)
        .max_len("name", &req.name, 64)
        .finish()?;

    let secret = admin_token::generate();
    let hash = admin_token::hash(&secret).map_err(AppError::Other)?;
    let perms = if req.permissions.is_empty() {
        vec![schema::PERM_ADMIN_ALL.to_string()]
    } else {
        req.permissions
    };
    let id = crate::db::create_admin_token(
        &state.db,
        &req.name,
        &admin_token::lookup(&secret),
        &hash,
        &perms,
    )
    .await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::ADMIN_TOKEN_CREATE,
        target("admin_token", id),
        json!({ "name": req.name, "permissions": perms }),
    )
    .await;
    Ok(created(
        format!("/api/admin/tokens/{id}"),
        json!({ "id": id, "token": secret, "permissions": perms }),
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(schema::PERM_ADMIN_ALL)?;
    crate::db::delete_admin_token(&state.db, id).await?;
    audit::record(
        &state,
        &admin.actor,
        audit::actions::ADMIN_TOKEN_DELETE,
        target("admin_token", id),
        json!({}),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}
