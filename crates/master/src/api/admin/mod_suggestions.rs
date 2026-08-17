use crate::api::auth::{AdminAuth, AuthUser};
use crate::db::models::ModSuggestionRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use schema::{OptionalMod, PERM_MODS_VIEW};

use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateSuggestionReq {
    pub server_id: Uuid,
    pub build_id: Option<Uuid>,
    pub provider: String,
    pub project_id: String,
    pub title: String,
    pub icon_url: Option<String>,
    pub description: Option<String>,
}

/// Игрок в лаунчере отправляет предложение опционального мода.
pub async fn create_suggestion(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateSuggestionReq>,
) -> AppResult<Json<ModSuggestionRow>> {
    let build_id = match req.build_id {
        Some(bid) => Some(bid),
        None => crate::db::list_builds(&state.db, req.server_id)
            .await
            .ok()
            .and_then(|builds| builds.first().map(|b| b.id)),
    };

    let row = crate::db::create_mod_suggestion(
        &state.db,
        req.server_id,
        build_id,
        &req.provider,
        &req.project_id,
        &req.title,
        req.icon_url.as_deref(),
        req.description.as_deref(),
        user.user_id,
    )
    .await?;

    Ok(Json(row))
}

#[derive(Deserialize)]
pub struct ListSuggestionsQuery {
    pub server_id: Uuid,
    pub status: Option<String>,
}

/// Админ получает список заявок на добавление модов.
pub async fn list_suggestions(
    State(state): State<AppState>,
    admin: AdminAuth,
    Query(q): Query<ListSuggestionsQuery>,
) -> AppResult<Json<Vec<ModSuggestionRow>>> {
    admin.require(PERM_MODS_VIEW)?;
    let rows = crate::db::list_mod_suggestions(&state.db, q.server_id, q.status.as_deref()).await?;
    Ok(Json(rows))
}

/// Админ одобряет заявку: мод добавляется в optional_mods сборки.
pub async fn approve_suggestion(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_MODS_VIEW)?;
    let suggestion = crate::db::get_mod_suggestion(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("request".into()))?;

    if suggestion.status == "approved" {
        return Ok(Json(json!({ "ok": true, "already_approved": true })));
    }

    let build_id = match suggestion.build_id {
        Some(bid) => Some(bid),
        None => crate::db::list_builds(&state.db, suggestion.server_id)
            .await
            .ok()
            .and_then(|builds| builds.first().map(|b| b.id)),
    };

    if let Some(bid) = build_id {
        if let Some(build) = crate::db::get_build(&state.db, bid).await? {
            let mut opts: Vec<OptionalMod> =
                serde_json::from_value(build.optional_mods.clone()).unwrap_or_default();

            if !opts.iter().any(|m| m.name == suggestion.title) {
                opts.push(OptionalMod {
                    name: suggestion.title.clone(),
                    description: suggestion.description.clone().unwrap_or_default(),
                    category: "Suggested".into(),
                    files: vec![],
                    enabled_by_default: false,
                    visible: true,
                    limited: false,
                    dependencies: vec![],
                    conflicts: vec![],
                    triggers: vec![],
                    icon_url: suggestion.icon_url.clone(),
                    author: None,
                });

                let new_opts = serde_json::to_value(opts).map_err(|e| AppError::Other(e.into()))?;
                sqlx::query("UPDATE builds SET optional_mods = $1 WHERE id = $2")
                    .bind(new_opts)
                    .bind(bid)
                    .execute(&state.db)
                    .await?;
            }
        }
    }

    let updated = crate::db::update_mod_suggestion_status(&state.db, id, "approved").await?;
    Ok(Json(json!({ "ok": true, "suggestion": updated })))
}

#[derive(Deserialize)]
pub struct AcceptSuggestionReq {
    /// "optional" — add to optional_mods list; "regular" — add as normal build file.
    pub mode: String,
    /// When mode="regular", whether to also install on game servers.
    #[serde(default)]
    pub install_on_servers: bool,
}

/// Админ принимает заявку с выбором: как опциональный или как обычный мод.
pub async fn accept_suggestion(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<AcceptSuggestionReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_MODS_VIEW)?;
    let suggestion = crate::db::get_mod_suggestion(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("request".into()))?;

    if suggestion.status == "approved" {
        return Ok(Json(json!({ "ok": true, "already_approved": true })));
    }

    let bid = match suggestion.build_id {
        Some(bid) => bid,
        None => {
            let builds = crate::db::list_builds(&state.db, suggestion.server_id).await?;
            builds
                .first()
                .map(|b| b.id)
                .ok_or_else(|| AppError::BadRequest("the server has no available builds".into()))?
        }
    };
    let build = crate::db::get_build(&state.db, bid)
        .await?
        .ok_or_else(|| AppError::NotFound("build".into()))?;

    let source = crate::catalog::source_for_project(
        &state,
        &suggestion.provider,
        &suggestion.project_id,
        &build.mc_version,
        &build.modloader,
    )
    .await?;

    let resolved = crate::catalog::resolve::resolve(&state, &source).await?;
    let path = format!("mods/{}", resolved.filename);

    crate::db::upsert_build_file(
        &state.db,
        bid,
        &path,
        &resolved.sha1,
        resolved.size as i64,
        "both",
        "mod",
    )
    .await?;

    if req.mode == "optional" {
        let entry = OptionalMod {
            name: suggestion.title.clone(),
            description: suggestion.description.clone().unwrap_or_default(),
            category: "Suggested".into(),
            files: vec![path.clone()],
            enabled_by_default: false,
            visible: true,
            limited: false,
            dependencies: vec![],
            conflicts: vec![],
            triggers: vec![],
            icon_url: resolved.icon_url.clone().or(suggestion.icon_url.clone()),
            author: resolved.author.clone(),
        };
        super::builds::append_optional_mod(&state, bid, entry).await?;
    }

    if req.install_on_servers {
        let servers = crate::db::list_game_servers(&state.db, build.server_id).await?;
        for gs in servers {
            if !gs.is_proxy() {
                let _ = crate::wrapper::ops::install_mod(&state, gs.id, &resolved).await;
            }
        }
    }

    super::builds::broadcast_builds_changed(&state, build.server_id);
    crate::db::update_mod_suggestion_status(&state.db, id, "approved").await?;
    Ok(Json(json!({ "ok": true, "mode": req.mode, "path": path })))
}

/// Админ отклоняет заявку.
pub async fn reject_suggestion(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_MODS_VIEW)?;
    let updated = crate::db::update_mod_suggestion_status(&state.db, id, "rejected").await?;
    Ok(Json(json!({ "ok": true, "suggestion": updated })))
}
