//! Пишущие методы: PUT, DELETE, MKCOL, MOVE.
//!
//! Все они делают ровно то же, что файловый менеджер в админке: правят
//! `build_files` и рассылают `BuildsChanged`. Манифест переподписывает Publish.

use super::{status, tree};
use crate::db::models::BuildFileRow;
use crate::state::AppState;
use axum::body::Body;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::Response;
use uuid::Uuid;

/// Потолок на файл: DAV — это про конфиги, моды заливают через админку.
const MAX_BYTES: usize = 32 * 1024 * 1024;

pub async fn put(state: &AppState, build_id: Uuid, path: &str, body: Body) -> Response {
    if path.is_empty() {
        return status(StatusCode::CONFLICT);
    }
    let data = match axum::body::to_bytes(body, MAX_BYTES).await {
        Ok(data) => data,
        Err(_) => return status(StatusCode::PAYLOAD_TOO_LARGE),
    };
    let Ok(stored) = state.files.put_bytes(&data).await else {
        return status(StatusCode::INTERNAL_SERVER_ERROR);
    };
    if store(state, build_id, path, &stored.sha1, stored.size as i64)
        .await
        .is_err()
    {
        return status(StatusCode::INTERNAL_SERVER_ERROR);
    }
    notify(state, build_id).await;
    status(StatusCode::CREATED)
}

pub async fn delete(
    state: &AppState,
    build_id: Uuid,
    files: &[BuildFileRow],
    path: &str,
) -> Response {
    let targets = tree::under(files, path);
    if targets.is_empty() {
        return status(StatusCode::NOT_FOUND);
    }
    for file in targets {
        if crate::db::delete_build_file(&state.db, file.id).await.is_err() {
            return status(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    notify(state, build_id).await;
    status(StatusCode::NO_CONTENT)
}

/// Каталогов в базе нет, они существуют через пути файлов. Создавать нечего,
/// но отказывать нельзя: Finder делает MKCOL перед первым файлом в новой папке.
pub fn mkcol(files: &[BuildFileRow], path: &str) -> Response {
    if tree::find(files, path).is_some() {
        return status(StatusCode::METHOD_NOT_ALLOWED);
    }
    status(StatusCode::CREATED)
}

pub async fn mv(
    state: &AppState,
    build_id: Uuid,
    files: &[BuildFileRow],
    base: &str,
    path: &str,
    parts: &Parts,
) -> Response {
    let Some(destination) = destination(base, parts) else {
        return status(StatusCode::BAD_REQUEST);
    };
    let targets = tree::under(files, path);
    if targets.is_empty() {
        return status(StatusCode::NOT_FOUND);
    }

    for file in targets {
        // Переносим и файл, и целое поддерево: у потомков меняется только префикс.
        let moved = match file.path.strip_prefix(path) {
            Some(tail) => format!("{destination}{tail}"),
            None => destination.clone(),
        };
        if store(state, build_id, &moved, &file.sha1, file.size).await.is_err()
            || crate::db::delete_build_file(&state.db, file.id).await.is_err()
        {
            return status(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    notify(state, build_id).await;
    status(StatusCode::CREATED)
}

/// `Destination` приходит абсолютным URL; нам нужен путь внутри той же сборки.
fn destination(base: &str, parts: &Parts) -> Option<String> {
    let raw = parts.headers.get("Destination")?.to_str().ok()?;
    let path = raw.split_once("://").map_or(raw, |(_, rest)| {
        rest.split_once('/').map_or("", |(_, tail)| tail)
    });
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    let rest = path.strip_prefix(base)?;
    tree::normalize(&urlencoding::decode(rest).ok()?)
}

async fn store(
    state: &AppState,
    build_id: Uuid,
    path: &str,
    sha1: &str,
    size: i64,
) -> anyhow::Result<()> {
    let kind = if path.starts_with("mods/") {
        "mod"
    } else if path.starts_with("config/") || path.ends_with(".toml") || path.ends_with(".json") {
        "config"
    } else {
        "other"
    };
    crate::db::upsert_build_file(&state.db, build_id, path, sha1, size, "both", kind).await
}

/// Тот же сигнал, что шлёт файловый менеджер: админки обновят дерево.
async fn notify(state: &AppState, build_id: Uuid) {
    if let Ok(Some(build)) = crate::db::get_build(&state.db, build_id).await {
        state
            .ws
            .broadcast(&schema::ServerWsMsg::BuildsChanged {
                server_id: build.server_id,
            });
    }
}
