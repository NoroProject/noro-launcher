//! Предпросмотр плашки роли: та же картинка, что уедет в игру.
//!
//! Отдельным маршрутом, а не полем в JSON роли: плашка это PNG, и рисовать её
//! в вебе заново значило бы завести вторую реализацию шрифта — она разъехалась
//! бы с настоящей на первой же правке. Здесь админка видит ровно то, что увидит
//! игрок.

use crate::api::auth::AdminAuth;
use crate::error::AppResult;
use crate::prefix;
use axum::extract::Query;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::IntoResponse;
use schema::PERM_ROLES_VIEW;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Ask {
    /// Что написать. Пусто — покажем название роли, как и в игре.
    #[serde(default)]
    pub text: String,
    /// Цвет роли, `#rrggbb`. Мусор превратится в серый, а не в ошибку.
    #[serde(default)]
    pub color: String,
    /// Ник рядом с плашкой. Пусто — только плашка.
    #[serde(default)]
    pub name: String,
}

/// `GET /api/admin/prefix-badge?text=…&color=…`
///
/// Параметрами, а не по идентификатору роли: так предпросмотр обновляется прямо
/// во время набора, до сохранения.
pub async fn badge(_admin: AdminAuth, Query(ask): Query<Ask>) -> AppResult<impl IntoResponse> {
    _admin.require(PERM_ROLES_VIEW)?;
    let text = if ask.text.trim().is_empty() {
        "ROLE"
    } else {
        ask.text.trim()
    };
    let png = if ask.name.trim().is_empty() {
        prefix::badge_png(text, &ask.color)?
    } else {
        prefix::badge_line(text, &ask.color, ask.name.trim())?
    };
    Ok((
        [
            (CONTENT_TYPE, "image/png"),
            // Картинка меняется вместе с полем, поэтому не кэшируем: иначе
            // предпросмотр застынет на первом варианте.
            (CACHE_CONTROL, "no-store"),
        ],
        png,
    ))
}
