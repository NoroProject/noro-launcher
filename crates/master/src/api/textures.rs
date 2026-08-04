//! Встроенные текстуры по умолчанию.
//!
//! Скин Стива отдаётся всем, у кого нет своего: без него кабинет и превью в
//! лаунчере показывали пустоту, хотя в самой игре клиент подставляет Стива сам.

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

/// Классический шаблон скина Mojang, 64×64.
const STEVE: &[u8] = include_bytes!("../../assets/steve.png");

/// Содержимое неизменяемо и вшито в бинарник, поэтому кешируется навсегда.
pub async fn default_skin() -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        STEVE,
    )
        .into_response()
}
