//! Раздача файлов из content-addressable стора по SHA1.
//!
//! URL содержит хеш содержимого, поэтому ответ иммутабелен: по одному URL
//! никогда не может прийти другой контент. Отсюда `immutable` в Cache-Control
//! и ETag из самого SHA1 — он стабилен между репликами мастера, в отличие от
//! ETag по mtime, который развалил бы кеш CDN при нескольких origin'ах.
//!
//! Range нужен и CDN (сегментация крупных файлов), и лаунчеру (докачка).

use super::range::{header_has, wanted, Wanted};
use crate::state::AppState;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// Имя, под которым файл должен сохраниться у пользователя.
///
/// URL контент-адресный, поэтому по умолчанию браузер называет скачанное хешем.
/// HTML-атрибут `download` тут не спасает: админка и мастер — разные origin, и
/// он в этом случае игнорируется. Значит имя должен назвать сам сервер.
#[derive(serde::Deserialize, Default)]
pub struct DownloadName {
    pub name: Option<String>,
}

pub async fn serve_file(
    State(state): State<AppState>,
    Path(sha1): Path<String>,
    Query(download): Query<DownloadName>,
    headers: HeaderMap,
) -> Response {
    if sha1.len() < 4 || !sha1.bytes().all(|b| b.is_ascii_hexdigit()) {
        return (StatusCode::BAD_REQUEST, "invalid sha1").into_response();
    }
    let Ok(mut file) = state.files.open(&sha1).await else {
        return (StatusCode::NOT_FOUND, "file not found").into_response();
    };
    let Ok(meta) = file.metadata().await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "stat failed").into_response();
    };
    let total = meta.len();
    let etag = format!("\"{sha1}\"");
    let filename = download.name.as_deref().and_then(safe_filename);

    let mut magic = [0u8; 12];
    let content_type = if total >= 12 && file.read_exact(&mut magic).await.is_ok() {
        sniff_content_type(&magic)
    } else {
        "application/octet-stream"
    };
    // Не «по возможности»: без возврата в начало отдалась бы копия без первых
    // восьми байт — с кодом 200 и правильным ETag. Лаунчер такой файл принял бы,
    // не сошёлся по sha1 и решил, что сборку подменили.
    if let Err(e) = file.seek(std::io::SeekFrom::Start(0)).await {
        tracing::error!(%sha1, error = %e, "не перемотать файл в начало");
        return (StatusCode::INTERNAL_SERVER_ERROR, "seek failed").into_response();
    }

    // Совпавший ETag на content-addressed URL не может быть протухшим.
    if header_has(&headers, header::IF_NONE_MATCH, &etag) {
        return base(&etag, content_type)
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .unwrap();
    }

    let base = |etag: &str| with_name(base(etag, content_type), filename.as_deref());

    match wanted(&headers, &etag, total) {
        Wanted::Unsatisfiable => base(&etag)
            .status(StatusCode::RANGE_NOT_SATISFIABLE)
            .header(header::CONTENT_RANGE, format!("bytes */{total}"))
            .body(Body::empty())
            .unwrap(),

        Wanted::Whole => {
            let stream = ReaderStream::new(file);
            base(&etag)
                .header(header::CONTENT_LENGTH, total)
                .body(Body::from_stream(stream))
                .unwrap()
        }

        Wanted::Part { start, end } => {
            if file.seek(std::io::SeekFrom::Start(start)).await.is_err() {
                return (StatusCode::INTERNAL_SERVER_ERROR, "seek failed").into_response();
            }
            let len = end - start + 1;
            let stream = ReaderStream::new(file.take(len));
            base(&etag)
                .status(StatusCode::PARTIAL_CONTENT)
                .header(header::CONTENT_LENGTH, len)
                .header(
                    header::CONTENT_RANGE,
                    format!("bytes {start}-{end}/{total}"),
                )
                .body(Body::from_stream(stream))
                .unwrap()
        }
    }
}

/// Только то, из чего нельзя собрать инъекцию в заголовок: перевод строки или
/// кавычка в имени файла разъехались бы по Content-Disposition.
fn safe_filename(raw: &str) -> Option<String> {
    let ok = !raw.is_empty()
        && raw.len() <= 128
        && raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_');
    ok.then(|| raw.to_string())
}

fn with_name(
    builder: axum::http::response::Builder,
    filename: Option<&str>,
) -> axum::http::response::Builder {
    match filename {
        Some(name) => builder.header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{name}\""),
        ),
        None => builder,
    }
}

/// Заголовки, общие для всех ответов: кеш, валидатор, поддержка Range.
/// Тип картинки по первым байтам.
///
/// Раньше узнавался только PNG, а всё остальное уезжало как
/// `application/octet-stream`. Для анимации это важнее, чем кажется: браузер
/// без типа полагается на угадывание, и часть окружений (в том числе строгий
/// CSP и `<img>` в Safari) такую картинку просто не показывает.
fn sniff_content_type(magic: &[u8]) -> &'static str {
    match magic {
        m if m.starts_with(b"\x89PNG\r\n\x1a\n") => "image/png",
        m if m.starts_with(b"GIF87a") || m.starts_with(b"GIF89a") => "image/gif",
        m if m.starts_with(b"\xff\xd8\xff") => "image/jpeg",
        m if m.len() >= 12 && m.starts_with(b"RIFF") && &m[8..12] == b"WEBP" => "image/webp",
        _ => "application/octet-stream",
    }
}

fn base(etag: &str, content_type: &str) -> axum::http::response::Builder {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, IMMUTABLE)
        .header(header::ETAG, etag)
        .header(header::ACCEPT_RANGES, "bytes")
}
