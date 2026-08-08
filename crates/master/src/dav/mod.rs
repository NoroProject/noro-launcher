//! WebDAV поверх файлов сборки: `/dav/{build_id}/…`.
//!
//! Монтируется в Finder и Explorer как диск, чтобы править конфиги обычным
//! редактором. Никакого своего порта и своих учёток: тот же HTTP-сервер и тот
//! же админ-токен, только через Basic — Finder Bearer не умеет.
//!
//! Запись ведёт себя как файловый менеджер: обновляет `build_files` и рассылает
//! `BuildsChanged`. Манифест **не** переподписывается — это отдельное действие
//! Publish, и правильно: иначе каждое сохранение конфига поднимало бы клиентам
//! новую версию сборки.

mod dispatch;
mod propfind;
mod read;
mod tree;
mod write;

use crate::api::auth::AdminAuth;
use crate::state::AppState;
use axum::body::Body;
use axum::extract::{FromRequestParts, Request, State};
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::Router;
use base64::Engine;
use uuid::Uuid;

/// Методы сверх обычного HTTP, которые понимает наш DAV.
const ALLOW: &str =
    "OPTIONS, GET, HEAD, PROPFIND, PROPPATCH, PUT, DELETE, MKCOL, MOVE, LOCK, UNLOCK";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dav/{build_id}", any(handle))
        // Отдельным маршрутом: axum считает путь со слэшем на конце другим, а
        // Finder обращается к каталогам именно так.
        .route("/dav/{build_id}/", any(handle))
        .route("/dav/{build_id}/{*path}", any(handle))
}

async fn handle(State(state): State<AppState>, request: Request) -> Response {
    // OPTIONS обязан отвечать до авторизации: Finder сначала спрашивает
    // возможности сервера и только потом присылает учётные данные.
    if request.method() == Method::OPTIONS {
        return options();
    }

    let (mut parts, body) = request.into_parts();
    if let Err(response) = authorize(&state, &mut parts).await {
        return response;
    }

    let build_id = match parts
        .uri
        .path()
        .trim_start_matches("/dav/")
        .split('/')
        .next()
        .and_then(|raw| Uuid::parse_str(raw).ok())
    {
        Some(id) => id,
        None => return status(StatusCode::NOT_FOUND),
    };

    let base = format!("/dav/{build_id}");
    let rest = parts.uri.path().trim_start_matches(&base);
    let Some(path) = tree::normalize(&urlencoding::decode(rest).unwrap_or_default()) else {
        return status(StatusCode::FORBIDDEN);
    };

    dispatch::route(&state, build_id, &base, &path, parts, body).await
}

fn options() -> Response {
    Response::builder()
        // DAV: 2 — обязательное условие, чтобы Finder смонтировал том на запись:
        // без поддержки блокировок он молча переходит в режим только чтения.
        .header("DAV", "1, 2")
        .header(header::ALLOW, ALLOW)
        .header("MS-Author-Via", "DAV")
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()
}

/// Basic → тот же самый `AdminAuth`, что и у остального админского API.
///
/// Логин игнорируется, паролем идёт админ-токен: так его можно вставить в
/// стандартный диалог подключения к серверу, не заводя второе хранилище учёток.
async fn authorize(
    state: &AppState,
    parts: &mut axum::http::request::Parts,
) -> Result<(), Response> {
    let token = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Basic "))
        .and_then(|encoded| {
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .ok()
        })
        .and_then(|raw| String::from_utf8(raw).ok())
        .map(|pair| {
            pair.split_once(':')
                .map(|(_, pass)| pass.to_string())
                .unwrap_or(pair)
        });

    let Some(token) = token else {
        return Err(unauthorized());
    };

    // Подменяем заголовок на Bearer и отдаём штатному экстрактору — дублировать
    // разбор токенов и проверку прав здесь нельзя, разъедется.
    if let Ok(value) = HeaderValue::from_str(&format!("Bearer {token}")) {
        parts.headers.insert(header::AUTHORIZATION, value);
    }
    match <AdminAuth as FromRequestParts<AppState>>::from_request_parts(parts, state).await {
        Ok(admin) if admin.require(schema::PERM_ADMIN_BUILDS).is_ok() => Ok(()),
        _ => Err(unauthorized()),
    }
}

fn unauthorized() -> Response {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::WWW_AUTHENTICATE, r#"Basic realm="noro builds""#)
        .body(Body::empty())
        .unwrap()
}

pub(crate) fn status(code: StatusCode) -> Response {
    (code, "").into_response()
}
