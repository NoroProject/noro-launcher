//! Разбор заголовка `Range` для раздачи файлов.

use axum::http::{header, HeaderMap};

/// Разобранный заголовок Range.
pub enum Wanted {
    /// Range не запрошен либо проигнорирован (не тот If-Range).
    Whole,
    /// Полуинтервал включительно: `[start, end]`.
    Part {
        start: u64,
        end: u64,
    },
    Unsatisfiable,
}

pub fn header_has(headers: &HeaderMap, name: header::HeaderName, etag: &str) -> bool {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == "*" || v.split(',').any(|t| t.trim() == etag))
}

pub fn wanted(headers: &HeaderMap, etag: &str, total: u64) -> Wanted {
    let Some(raw) = headers.get(header::RANGE).and_then(|v| v.to_str().ok()) else {
        return Wanted::Whole;
    };
    // If-Range с другим валидатором означает «отдай целиком».
    if headers.contains_key(header::IF_RANGE) && !header_has(headers, header::IF_RANGE, etag) {
        return Wanted::Whole;
    }
    parse_range(raw, total)
}

/// Единичный `bytes=`-диапазон. Мультидиапазоны отдаются целиком — лаунчеру и
/// CDN они не нужны, а multipart-ответ стоил бы заметной сложности.
fn parse_range(raw: &str, total: u64) -> Wanted {
    let Some(spec) = raw.trim().strip_prefix("bytes=") else {
        return Wanted::Whole;
    };
    if spec.contains(',') {
        return Wanted::Whole;
    }
    let Some((from, to)) = spec.split_once('-') else {
        return Wanted::Unsatisfiable;
    };
    let (from, to) = (from.trim(), to.trim());

    let (start, end) = if from.is_empty() {
        // `bytes=-N` — последние N байт.
        match to.parse::<u64>() {
            Ok(0) | Err(_) => return Wanted::Unsatisfiable,
            Ok(n) => (total.saturating_sub(n), total - 1),
        }
    } else {
        let Ok(start) = from.parse::<u64>() else {
            return Wanted::Unsatisfiable;
        };
        let end = if to.is_empty() {
            total.saturating_sub(1)
        } else {
            match to.parse::<u64>() {
                Ok(e) => e.min(total.saturating_sub(1)),
                Err(_) => return Wanted::Unsatisfiable,
            }
        };
        (start, end)
    };

    if total == 0 || start >= total || start > end {
        Wanted::Unsatisfiable
    } else {
        Wanted::Part { start, end }
    }
}

#[cfg(test)]
#[path = "range_tests.rs"]
mod tests;
