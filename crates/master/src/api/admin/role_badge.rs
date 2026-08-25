//! Своя картинка плашки роли.
//!
//! Нарисованная нами плашка подходит не всем: у роли может быть свой знак,
//! которого шрифтом 5×7 не набрать. Загруженная картинка заменяет её целиком —
//! ни градиент, ни текст к ней уже не подмешиваются.
//!
//! Требования жёсткие намеренно. Плашка живёт в строке чата рядом с ником, и
//! картинка не того размера либо разъедет строку, либо окажется обрезанной: в
//! шрифте она масштабируется по высоте, а не вписывается.

use crate::api::auth::AdminAuth;
use crate::error::{AppError, AppResult};
use crate::prefix;
use crate::state::AppState;
use axum::extract::{Multipart, Path, State};
use axum::Json;
use schema::PERM_ROLES_EDIT;
use serde_json::{json, Value};
use uuid::Uuid;

/// Во сколько раз картинка может быть выше строки.
///
/// Ровно в высоту плашки — тесно: рисовать в семь пикселей руками неудобно.
/// Кратность даёт запас, а в игре картинка ужмётся обратно без мыла: каждый
/// пиксель схлопнется в один.
const MAX_SCALE: u32 = 8;

/// Дальше плашка занимает пол-экрана и вытесняет сам разговор.
const MAX_RATIO: u32 = 12;

/// `PUT /api/admin/roles/{id}/badge`
pub async fn upload(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
    multipart: Multipart,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ROLES_EDIT)?;
    let bytes = read_file(multipart).await?;
    let (width, height) = measure(&bytes)?;

    let stored = state
        .files
        .put_bytes(&bytes)
        .await
        .map_err(AppError::Other)?;
    sqlx::query("UPDATE roles SET badge_sha1 = $2 WHERE id = $1")
        .bind(id)
        .bind(&stored.sha1)
        .execute(&state.db)
        .await?;

    Ok(Json(
        json!({ "sha1": stored.sha1, "width": width, "height": height }),
    ))
}

/// `DELETE /api/admin/roles/{id}/badge` — вернуться к нарисованной плашке.
pub async fn clear(
    State(state): State<AppState>,
    admin: AdminAuth,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    admin.require(PERM_ROLES_EDIT)?;
    sqlx::query("UPDATE roles SET badge_sha1 = NULL WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

async fn read_file(mut multipart: Multipart) -> AppResult<Vec<u8>> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if field.name() == Some("file") {
            return Ok(field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?
                .to_vec());
        }
    }
    Err(AppError::bad(
        crate::error_codes::UPLOAD_FIELD_MISSING,
        "прикрепите картинку",
    ))
}

/// Проверить размеры и вернуть их.
///
/// Отказ подробный, а не «неверный файл»: человек должен понять, что именно
/// поправить, не гадая.
pub(crate) fn measure(bytes: &[u8]) -> AppResult<(u32, u32)> {
    let image = image::load_from_memory(bytes)
        .map_err(|_| AppError::bad(crate::error_codes::UPLOAD_BAD_FORMAT, "это не картинка"))?;
    let (width, height) = (image.width(), image.height());
    let line = prefix::badge_height() as u32;

    if height == 0 || height % line != 0 {
        return Err(AppError::BadRequest(format!(
            "высота должна быть кратна {line}: {line}, {}, {}…, а не {height}",
            line * 2,
            line * 3
        )));
    }
    let scale = height / line;
    if scale > MAX_SCALE {
        return Err(AppError::BadRequest(format!(
            "слишком высокая: не больше {}",
            line * MAX_SCALE
        )));
    }
    if width == 0 || width > height * MAX_RATIO {
        return Err(AppError::BadRequest(format!(
            "слишком широкая: при высоте {height} ширина до {}",
            height * MAX_RATIO
        )));
    }
    Ok((width, height))
}
