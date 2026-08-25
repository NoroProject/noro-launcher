//! Админ: OAuth2-приложения инстанса.
//!
//! Оператор решает три вопроса: пускать ли приложение к чужим аккаунтам
//! (модерация), насколько глубоко пускать (привилегированные scope'ы) и
//! работают ли сторонние приложения вообще (выключатель).

use crate::api::auth::AdminAuth;
use crate::audit::{self, target};
use crate::config::keys;
use crate::db::oauth_apps::{self, OAuthApp};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use schema::{PERM_OAUTH_MANAGE, PERM_OAUTH_VIEW};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
}

/// GET /api/admin/oauth-apps — все приложения инстанса.
pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_OAUTH_VIEW)?;
    let status = q.status.as_deref().filter(|s| !s.is_empty());
    if let Some(s) = status {
        if !oauth_apps::ALL_STATUSES.contains(&s) {
            return Err(AppError::BadRequest(format!("unknown status: {s}")));
        }
    }

    let apps = crate::db::list_apps(&state.db, status).await?;
    let counts: BTreeMap<Uuid, i64> = crate::db::authorization_counts(&state.db)
        .await?
        .into_iter()
        .collect();

    // Имена владельцев одним проходом: список открывают, чтобы понять, кто
    // просится, и UUID в этом вопросе не помогает.
    let mut items = Vec::with_capacity(apps.len());
    for app in &apps {
        let owner = match app.owner_id {
            Some(id) => crate::db::get_user(&state.db, id)
                .await?
                .map(|u| json!({ "id": u.id, "username": u.mc_username })),
            None => None,
        };
        items.push(view(app, owner, counts.get(&app.id).copied().unwrap_or(0)));
    }

    Ok(Json(json!({
        "items": items,
        "pending": apps
            .iter()
            .filter(|a| a.status == oauth_apps::STATUS_PENDING)
            .count(),
        "scopes": schema::ALL_SCOPES
            .iter()
            .filter(|s| s.tier != schema::ScopeTier::Internal)
            .map(|s| json!({ "name": s.name, "title": s.title, "tier": s.tier }))
            .collect::<Vec<_>>(),
        "apps_enabled": state.oauth_apps_enabled().await,
        "creation_enabled": state.oauth_apps_creation_enabled().await,
    })))
}

#[derive(Deserialize)]
pub struct StatusReq {
    pub status: String,
    /// Что увидит автор. Для отказа обязательна: «нет» без причины возвращает
    /// человека с тем же приложением на следующий день.
    #[serde(default)]
    pub note: Option<String>,
}

/// PUT /api/admin/oauth-apps/{id}/status — решение по приложению.
pub async fn set_status(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<StatusReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_OAUTH_MANAGE)?;
    if !oauth_apps::ALL_STATUSES.contains(&req.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "unknown status: {}",
            req.status
        )));
    }
    let app = load(&state, id).await?;
    if app.is_official {
        return Err(AppError::BadRequest(
            "official applications are not moderated".into(),
        ));
    }
    let note = req.note.as_deref().map(str::trim).filter(|n| !n.is_empty());
    if req.status == oauth_apps::STATUS_REJECTED && note.is_none() {
        return Err(AppError::BadRequest(
            "a rejection needs a reason for the author".into(),
        ));
    }

    let updated = crate::db::set_status(&state.db, id, &req.status, note, admin.user_id())
        .await?
        .ok_or_else(|| AppError::NotFound("application".into()))?;

    // Заблокированное или отклонённое приложение теряет и уже выданные токены:
    // иначе запрет означал бы только «новых игроков не пускать».
    if matches!(
        req.status.as_str(),
        oauth_apps::STATUS_REJECTED | oauth_apps::STATUS_SUSPENDED
    ) {
        crate::db::revoke_all_sessions_for_app(&state.db, id).await?;
    }

    audit::record(
        &state,
        &admin.actor,
        crate::audit::actions::OAUTH_APP_REVIEW,
        target("oauth_app", id),
        json!({ "name": updated.name, "status": req.status, "note": note }),
    )
    .await;
    Ok(Json(view(&updated, None, 0)))
}

#[derive(Deserialize)]
pub struct ScopesReq {
    pub scopes: Vec<String>,
}

/// PUT /api/admin/oauth-apps/{id}/scopes — потолок доступа приложения.
pub async fn set_scopes(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<ScopesReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_OAUTH_MANAGE)?;
    for scope in &req.scopes {
        if schema::grantable(scope).is_none() {
            return Err(AppError::bad(
                crate::error_codes::OAUTH_BAD_SCOPE,
                format!("unknown scope: {scope}"),
            ));
        }
    }
    let scopes = req.scopes.join(" ");
    let updated = crate::db::set_allowed_scopes(&state.db, id, &scopes)
        .await?
        .ok_or_else(|| AppError::NotFound("application".into()))?;

    audit::record(
        &state,
        &admin.actor,
        crate::audit::actions::OAUTH_APP_SCOPES,
        target("oauth_app", id),
        json!({ "name": updated.name, "scopes": req.scopes }),
    )
    .await;
    Ok(Json(view(&updated, None, 0)))
}

/// DELETE /api/admin/oauth-apps/{id} — удалить чужое приложение.
pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_OAUTH_MANAGE)?;
    let app = load(&state, id).await?;
    crate::db::revoke_all_sessions_for_app(&state.db, id).await?;
    let removed = crate::db::delete_app(&state.db, id).await?;

    audit::record(
        &state,
        &admin.actor,
        crate::audit::actions::OAUTH_APP_DELETE,
        target("oauth_app", id),
        json!({ "name": app.name }),
    )
    .await;
    Ok(Json(json!({ "ok": removed })))
}

/// POST /api/admin/oauth-apps/{id}/icon — иконка любому приложению.
///
/// Нужна ради официальных: у них нет владельца, который зашёл бы в кабинет и
/// залил картинку сам, а на экране согласия наш лаунчер должен выглядеть как
/// наш лаунчер, а не как безымянный клиент.
pub async fn upload_icon(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    multipart: axum::extract::Multipart,
) -> AppResult<Json<Value>> {
    admin.require(PERM_OAUTH_MANAGE)?;
    let app = load(&state, id).await?;
    let url = crate::api::apps::store_icon(&state, app.id, multipart).await?;

    audit::record(
        &state,
        &admin.actor,
        crate::audit::actions::OAUTH_APP_SCOPES,
        target("oauth_app", id),
        json!({ "name": app.name, "icon_url": url }),
    )
    .await;
    Ok(Json(json!({ "icon_url": url })))
}

#[derive(Deserialize)]
pub struct TogglesReq {
    /// Работают ли сторонние приложения. Официальных не касается.
    pub apps_enabled: Option<bool>,
    /// Можно ли заводить новые.
    pub creation_enabled: Option<bool>,
}

/// PUT /api/admin/oauth-apps/settings — выключатели.
pub async fn set_toggles(
    State(state): State<AppState>,
    admin: AdminAuth,
    Json(req): Json<TogglesReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_OAUTH_MANAGE)?;
    let mut values: BTreeMap<String, Value> = BTreeMap::new();
    if let Some(v) = req.apps_enabled {
        values.insert(keys::OAUTH_APPS_ENABLED.to_string(), Value::Bool(v));
    }
    if let Some(v) = req.creation_enabled {
        values.insert(keys::OAUTH_APPS_CREATION.to_string(), Value::Bool(v));
    }
    if values.is_empty() {
        return Err(AppError::BadRequest("nothing to change".into()));
    }
    crate::db::set_settings(&state.db, &values, admin.user_id()).await?;

    audit::record(
        &state,
        &admin.actor,
        crate::audit::actions::OAUTH_APP_TOGGLE,
        None,
        json!(values),
    )
    .await;
    Ok(Json(json!({
        "apps_enabled": state.oauth_apps_enabled().await,
        "creation_enabled": state.oauth_apps_creation_enabled().await,
    })))
}

async fn load(state: &AppState, id: Uuid) -> AppResult<OAuthApp> {
    crate::db::app_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("application".into()))
}

fn view(app: &OAuthApp, owner: Option<Value>, authorized_users: i64) -> Value {
    json!({
        "id": app.id,
        "client_id": app.client_id,
        "name": app.name,
        "description": app.description,
        "icon_url": app.icon_url,
        "redirect_uris": app.redirect_list(),
        "status": app.status,
        "review_note": app.review_note,
        "allowed_scopes": app.allowed_scope_list(),
        "is_official": app.is_official,
        "owner": owner,
        "authorized_users": authorized_users,
        "created_at": app.created_at,
        "updated_at": app.updated_at,
        "reviewed_at": app.reviewed_at,
    })
}
