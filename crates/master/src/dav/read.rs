//! Читающие методы: PROPFIND, GET/HEAD, плюс заглушки LOCK и PROPPATCH.

use super::propfind::{multistatus, Resource};
use super::{status, tree};
use crate::db::models::BuildFileRow;
use crate::state::AppState;
use axum::body::Body;
use axum::http::request::Parts;
use axum::http::{header, StatusCode};
use axum::response::Response;
use tokio_util::io::ReaderStream;

const MULTISTATUS: u16 = 207;

pub fn propfind(files: &[BuildFileRow], base: &str, path: &str, parts: &Parts) -> Response {
    // Depth: 0 — только сам ресурс, 1 — он и его прямые потомки. Бесконечную
    // глубину не поддерживаем: на сборке в тысячи файлов это выстрел в ногу.
    let depth = parts
        .headers
        .get("Depth")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("1");

    let mut resources: Vec<Resource> = Vec::new();

    if let Some(file) = tree::find(files, path) {
        resources.push(Resource {
            path: path.to_string(),
            is_dir: false,
            size: file.size,
            etag: file.sha1.clone(),
        });
    } else if tree::is_dir(files, path) {
        resources.push(Resource {
            path: path.to_string(),
            is_dir: true,
            size: 0,
            etag: String::new(),
        });
        if depth != "0" {
            for node in tree::children(files, path) {
                resources.push(Resource::from_node(path, node));
            }
        }
    } else {
        return status(StatusCode::NOT_FOUND);
    }

    let xml = multistatus(base, &resources);
    Response::builder()
        .status(StatusCode::from_u16(MULTISTATUS).unwrap())
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .body(Body::from(xml))
        .unwrap()
}

pub async fn get(
    state: &AppState,
    files: &[BuildFileRow],
    path: &str,
    with_body: bool,
) -> Response {
    let Some(file) = tree::find(files, path) else {
        return status(StatusCode::NOT_FOUND);
    };
    let builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, file.size)
        .header(header::ETAG, format!("\"{}\"", file.sha1));

    if !with_body {
        return builder.body(Body::empty()).unwrap();
    }
    match state.files.open(&file.sha1).await {
        Ok(handle) => builder
            .body(Body::from_stream(ReaderStream::new(handle)))
            .unwrap(),
        // Строка в базе есть, а блоба нет — стор разъехался с индексом.
        Err(_) => status(StatusCode::NOT_FOUND),
    }
}

/// Ответ на LOCK: токен выдаём, но ничего не запираем.
///
/// Реальные блокировки здесь не нужны — правит один админ. Finder же без
/// поддержки LOCK монтирует том только на чтение, поэтому отказать нельзя.
pub fn lock(base: &str, path: &str) -> Response {
    let token = format!("opaquelocktoken:{}", uuid::Uuid::new_v4());
    let xml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?><D:prop xmlns:D="DAV:"><D:lockdiscovery><D:activelock>
<D:locktype><D:write/></D:locktype><D:lockscope><D:exclusive/></D:lockscope><D:depth>infinity</D:depth>
<D:timeout>Second-3600</D:timeout><D:locktoken><D:href>{token}</D:href></D:locktoken>
<D:lockroot><D:href>{base}/{path}</D:href></D:lockroot></D:activelock></D:lockdiscovery></D:prop>"#
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .header("Lock-Token", format!("<{token}>"))
        .body(Body::from(xml))
        .unwrap()
}

/// «Свойство записано» — на деле выброшено. Хранилища свойств у нас нет, а
/// Finder без этого ответа считает том сломанным.
pub fn proppatch(base: &str, path: &str) -> Response {
    let xml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?><D:multistatus xmlns:D="DAV:"><D:response>
<D:href>{base}/{path}</D:href><D:propstat><D:prop/><D:status>HTTP/1.1 200 OK</D:status>
</D:propstat></D:response></D:multistatus>"#
    );
    Response::builder()
        .status(StatusCode::from_u16(MULTISTATUS).unwrap())
        .header(header::CONTENT_TYPE, "application/xml; charset=utf-8")
        .body(Body::from(xml))
        .unwrap()
}
