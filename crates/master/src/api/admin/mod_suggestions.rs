use crate::api::auth::{AdminAuth, AuthUser};
use crate::db::models::ModSuggestionRow;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use schema::{OptionalMod, PERM_ADMIN_BUILDS};
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
    let row = crate::db::create_mod_suggestion(
        &state.db,
        req.server_id,
        req.build_id,
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
    admin.require(PERM_ADMIN_BUILDS)?;
    let rows = crate::db::list_mod_suggestions(&state.db, q.server_id, q.status.as_deref()).await?;
    Ok(Json(rows))
}

/// Админ одобряет заявку: мод добавляется в optional_mods сборки.
pub async fn approve_suggestion(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_BUILDS)?;
    let suggestion = crate::db::get_mod_suggestion(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("заявка".into()))?;

    if suggestion.status == "approved" {
        return Ok(Json(json!({ "ok": true, "already_approved": true })));
    }

    if let Some(bid) = suggestion.build_id {
        if let Some(build) = crate::db::get_build(&state.db, bid).await? {
            let mut opts: Vec<OptionalMod> = serde_json::from_value(build.optional_mods.clone()).unwrap_or_default();

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

/// Админ отклоняет заявку.
pub async fn reject_suggestion(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ADMIN_BUILDS)?;
    let updated = crate::db::update_mod_suggestion_status(&state.db, id, "rejected").await?;
    Ok(Json(json!({ "ok": true, "suggestion": updated })))
}
