//! Плашка роли картинкой — всем, а не только админам.
//!
//! Тот же значок, что игрок видит в чате, нужен и в кабинете, и в списке ролей.
//! Отдельно от админского предпросмотра: тот принимает произвольный текст и
//! цвет и потому закрыт правом, а здесь всё берётся из самой роли — показывать
//! это можно кому угодно, роли и так видны.
//!
//! Отдаётся либо загруженная картинка, либо нарисованная. Для страницы это одно
//! и то же: она ставит `<img>` и не знает, что внутри.

use crate::error::{AppError, AppResult};
use crate::prefix;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::IntoResponse;
use uuid::Uuid;

/// `GET /api/roles/{id}/badge.png`
pub async fn badge(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let role = crate::db::list_roles(&state.db)
        .await?
        .into_iter()
        .find(|r| r.id == id)
        .ok_or_else(|| AppError::NotFound("нет такой роли".into()))?;

    let uploaded: Option<String> = sqlx::query_scalar("SELECT badge_sha1 FROM roles WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

    let png = match uploaded {
        Some(sha1) => tokio::fs::read(state.files.path_for(&sha1)).await.ok(),
        None => None,
    };
    let png = match png {
        Some(bytes) => bytes,
        None => {
            let text = role
                .prefix
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(&role.display_name);
            prefix::badge_png(text, role.color.as_deref().unwrap_or(""))?
        }
    };

    Ok((
        [
            (CONTENT_TYPE, "image/png"),
            // Минута: плашка меняется правкой роли, и ждать её сутками незачем,
            // но и перерисовывать на каждую строку списка тоже.
            (CACHE_CONTROL, "public, max-age=60"),
        ],
        png,
    ))
}
