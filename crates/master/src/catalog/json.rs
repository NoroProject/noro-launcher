//! Мелкие читалки JSON.
//!
//! Каталоги отдают полусвободную форму: поле может отсутствовать, быть null или
//! пустой строкой. Разбирать это по месту — значит утонуть в `.and_then`.

use serde_json::Value;

pub fn text(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap_or_default().to_string()
}

/// Пустая строка — то же самое, что отсутствие поля: и Modrinth, и CurseForge
/// подставляют `""` там, где значения нет.
pub fn opt_text(v: &Value, key: &str) -> Option<String> {
    v[key]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

pub fn num(v: &Value, key: &str) -> u64 {
    v[key].as_u64().unwrap_or(0)
}

pub fn str_list(v: &Value, key: &str) -> Vec<String> {
    v[key]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

pub fn items(v: &Value, key: &str) -> Vec<Value> {
    v[key].as_array().cloned().unwrap_or_default()
}
