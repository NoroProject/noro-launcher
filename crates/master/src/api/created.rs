//! `201 Created` responses.

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// `201 Created` with the new resource's address in `Location`.
///
/// The address is the resource's canonical id, not necessarily something that
/// answers GET: for roles and tokens it is where PUT and DELETE live.
pub fn created<T: Serialize>(location: impl AsRef<str>, body: T) -> Response {
    let mut res = (StatusCode::CREATED, Json(body)).into_response();
    // Callers build the address from a constant prefix and a UUID, so there is
    // nothing unprintable to trip over.
    let value =
        HeaderValue::from_str(location.as_ref()).expect("address is a prefix plus a UUID");
    res.headers_mut().insert(header::LOCATION, value);
    res
}
