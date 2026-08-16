//! Админ: API-токены для CLI/CI.

use crate::api::auth::{hash_admin_token, AdminAuth};
use crate::audit::{self, target};
use crate::db::models::AdminTokenRow;
use crate::error::AppResult;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
) -> AppResult<Json<Vec<AdminTokenRow>>> {
    admin.require(schema::PERM_ADMIN_ALL)?;
    Ok(Json(crate::db::list_admin_tokens(&state.db).await?))
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
) -> AppResult<Json<Value>> {
    admin.require(schema::PERM_ADMIN_ALL)?;
    // Сгенерировать секрет.
    let secret = {
        use rand::Rng;
        let bytes: [u8; 32] = rand::thread_rng().gen();
        format!("noro_{}", hex::encode(bytes))
    };
    let hash = hash_admin_token(&secret);
    let perms = if req.permissions.is_empty() {
        vec![schema::PERM_ADMIN_ALL.to_string()]
    } else {
        req.permissions
    };
    let id = crate::db::create_admin_token(&state.db, &req.name, &hash, &perms).await?;
    audit::record(
        &state,
        &admin.actor,
        "admin_token.create",
        target("admin_token", id),
        json!({ "name": req.name, "permissions": perms }),
    )
    .await;
    Ok(Json(
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
        "admin_token.delete",
        target("admin_token", id),
        json!({}),
    )
    .await;
    Ok(Json(json!({ "ok": true })))
}
