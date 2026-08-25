//! Вложение в дело: кадр экрана, снятый модератором.
//!
//! Killaura скриншотом не докажешь, а «стоял в чужом доме с чужим сундуком» —
//! вполне. Файл ложится в тот же CAS, что и капы: у кадра из дела нет своего
//! жизненного цикла, повторный снимок того же экрана не займёт места дважды.
//!
//! Право — `noro.mod.cases.claim`: кто ведёт дело, тот и прикладывает. Отдельного
//! права нет намеренно, иначе оно бы просто дублировало замок.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::{Multipart, Path, State};
use axum::Json;
use bytes::Bytes;
use schema::PERM_CASES_CLAIM;
use serde_json::json;
use uuid::Uuid;

/// Экран 4K в PNG укладывается с запасом, а всё, что больше, — уже не кадр,
/// а чья-то ошибка. Ограничение честнее молчаливой обрезки.
const MAX_SHOT_BYTES: usize = 8 * 1024 * 1024;

/// Подпись под кадром: короткая строка вроде «сундук соседа».
const MAX_NOTE_CHARS: usize = 200;

/// POST /api/admin/cases/{id}/attachments
pub async fn upload(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    admin.require(PERM_CASES_CLAIM)?;
    if crate::db::get_case(&state.db, id).await?.is_none() {
        return Err(AppError::NotFound("case".into()));
    }

    let (data, note) = read_parts(multipart).await?;
    let data = validate_png(data)?;
    let stored = state
        .files
        .put_bytes(&data)
        .await
        .map_err(AppError::Other)?;
    let file_url = state.config.file_url(&stored.sha1);

    crate::cases::event(
        &state,
        id,
        admin.actor.id(),
        &admin.actor.label(),
        "game",
        "screenshot",
        json!({
            "file_url": file_url,
            "sha1": stored.sha1,
            "size": stored.size,
            "note": note,
        }),
    )
    .await?;

    Ok(Json(json!({ "ok": true, "file_url": file_url })))
}

async fn read_parts(mut multipart: Multipart) -> AppResult<(Option<Bytes>, String)> {
    let mut data: Option<Bytes> = None;
    let mut note = String::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        match field.name() {
            Some("note") => {
                note = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
            }
            Some("file") => {
                data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(e.to_string()))?,
                );
            }
            _ => {}
        }
    }
    Ok((data, note.chars().take(MAX_NOTE_CHARS).collect()))
}

fn validate_png(data: Option<Bytes>) -> AppResult<Bytes> {
    let data = data.ok_or_else(|| {
        AppError::bad(
            crate::error_codes::UPLOAD_FIELD_MISSING,
            "missing file field",
        )
    })?;
    if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(AppError::bad(
            crate::error_codes::UPLOAD_BAD_FORMAT,
            "PNG expected",
        ));
    }
    if data.len() > MAX_SHOT_BYTES {
        return Err(AppError::bad(
            crate::error_codes::UPLOAD_TOO_LARGE,
            "screenshot is too large",
        ));
    }
    Ok(data)
}
