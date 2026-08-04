//! Разбор метода запроса. Axum о PROPFIND и MOVE не знает, поэтому маршрут
//! один (`any`), а ветвление — здесь.

use super::{read, status, write};
use crate::state::AppState;
use axum::body::Body;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Response;
use uuid::Uuid;

pub async fn route(
    state: &AppState,
    build_id: Uuid,
    base: &str,
    path: &str,
    parts: Parts,
    body: Body,
) -> Response {
    let files = match crate::db::build_files(&state.db, build_id).await {
        Ok(files) => files,
        Err(_) => return status(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match parts.method.as_str() {
        "PROPFIND" => read::propfind(&files, base, path, &parts),
        "GET" => read::get(state, &files, path, true).await,
        "HEAD" => read::get(state, &files, path, false).await,
        // Блокировки настоящие нам не нужны — редактор здесь один, админ.
        // Но без ответа на LOCK Finder монтирует том только на чтение.
        "LOCK" => read::lock(base, path),
        "UNLOCK" => status(StatusCode::NO_CONTENT),
        // Finder любит проставлять свои свойства (даты, метки Spotlight).
        // Отвечаем «принято» и ничего не храним: своей базы свойств у нас нет.
        "PROPPATCH" => read::proppatch(base, path),
        "PUT" => write::put(state, build_id, path, body).await,
        "DELETE" => write::delete(state, build_id, &files, path).await,
        "MKCOL" => write::mkcol(&files, path),
        "MOVE" => write::mv(state, build_id, &files, base, path, &parts).await,
        _ => status(StatusCode::METHOD_NOT_ALLOWED),
    }
}
