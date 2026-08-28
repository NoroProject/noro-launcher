//! Notes and the launch journal on a player's card.

use crate::api::auth::AdminAuth;
use crate::api::paging::{Page, PageQuery};
use crate::db::notes::{NoteRow, PlaySessionRow};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use schema::{
    PERM_USERS_JOURNAL, PERM_USERS_NOTES_DELETE, PERM_USERS_NOTES_VIEW, PERM_USERS_NOTES_WRITE,
};

use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn list(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<NoteRow>>> {
    admin.require(PERM_USERS_NOTES_VIEW)?;
    let (items, total) = crate::db::list_notes(&state.db, id, page.limit(), page.offset()).await?;
    Ok(Json(Page::new(items, total)))
}

#[derive(Deserialize)]
pub struct AddReq {
    pub body: String,
}

pub async fn add(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    Json(req): Json<AddReq>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_USERS_NOTES_WRITE)?;
    let body = req.body.trim();
    if body.is_empty() {
        return Err(AppError::BadRequest("the note is empty".into()));
    }
    let row = crate::db::add_note(
        &state.db,
        id,
        admin.user_id(),
        &admin.actor.label(),
        &body.chars().take(4000).collect::<String>(),
    )
    .await?;
    // Deliberately not audited: the note already carries its author and date.
    Ok(Json(json!(row)))
}

pub async fn delete(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path((_id, note_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_USERS_NOTES_DELETE)?;
    if !crate::db::delete_note(&state.db, note_id).await? {
        return Err(AppError::NotFound("note".into()));
    }
    Ok(Json(json!({ "ok": true })))
}

/// Launch journal: which build and which mods the player came in with.
pub async fn play_sessions(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<PlaySessionRow>>> {
    admin.require(PERM_USERS_JOURNAL)?;
    Ok(Json(
        crate::db::list_play_sessions(&state.db, id, 50).await?,
    ))
}
