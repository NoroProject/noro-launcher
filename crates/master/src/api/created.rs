//! Ответ на удачное создание ресурса.
//!
//! Все создающие ручки отдавали 200 и тело с id. Клиенту приходилось знать
//! заранее, что именно эта ручка создаёт, и самому собирать адрес созданного из
//! шаблона: код ответа об этом не говорил, а заголовка с адресом не было вовсе.

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// `201 Created` с адресом созданного ресурса в `Location`.
///
/// Адрес — канонический идентификатор ресурса, а не обязательно то, что
/// отвечает на GET: у роли и токена по нему живут PUT и DELETE, и этого хватает,
/// чтобы клиент дальше обращался к созданному, ничего не додумывая.
pub fn created<T: Serialize>(location: impl AsRef<str>, body: T) -> Response {
    let mut res = (StatusCode::CREATED, Json(body)).into_response();
    // Без запасного пути: адрес собираем мы сами из константного префикса и
    // UUID, непечатных символов там взяться неоткуда. Тихо отдать 201 без
    // `Location` значило бы сказать клиенту «создано» и не сказать где.
    let value =
        HeaderValue::from_str(location.as_ref()).expect("адрес создаётся из префикса и UUID");
    res.headers_mut().insert(header::LOCATION, value);
    res
}
