//! Слияние конфигов по ключам вместо файла целиком.
//!
//! Файловый three-way (`merge.rs`) отвечает «кто менял файл». Здесь вопрос
//! мельче: игрок поправил одну строку, сервер — другую, и обе правки должны
//! ужиться. Без этого такой случай остаётся конфликтом, хотя спорить не о чем.
//!
//! Требует реальную копию исходного файла, а не только его хеш — поэтому
//! `.noro/base/` появляется только для тех путей, где режим включён.
//!
//! Форматы: `.properties` и построчные `.txt` вроде `options.txt`. JSON и TOML
//! сюда не входят намеренно: там значение может быть деревом, и «слияние по
//! ключам» перестаёт быть однозначным ровно там, где начинает быть нужным.

use std::collections::BTreeMap;
use std::path::Path;

mod base;

pub use base::{base_copy_path, remember_base};

/// Слить три версии по ключам.
///
/// Возвращает `None`, если разрешить нельзя: один и тот же ключ изменили обе
/// стороны по-разному. Такой файл остаётся обычным конфликтом.
pub fn merge_properties(mine: &str, base: &str, theirs: &str) -> Option<String> {
    let mine_map = parse(mine);
    let base_map = parse(base);
    let theirs_map = parse(theirs);

    let mut out: BTreeMap<&str, String> = BTreeMap::new();
    let keys: Vec<&str> = mine_map
        .keys()
        .chain(base_map.keys())
        .chain(theirs_map.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    for key in keys {
        let m = mine_map.get(key);
        let b = base_map.get(key);
        let t = theirs_map.get(key);

        match (m, b, t) {
            // Ключ удалён обеими сторонами либо не было и нет.
            (None, _, None) => {}
            // Игрок удалил, сервер не менял — уважаем удаление.
            (None, Some(b), Some(t)) if b == t => {}
            // Сервер удалил, игрок не менял.
            (Some(m), Some(b), None) if m == b => {}
            // Обе стороны согласны.
            (Some(m), _, Some(t)) if m == t => {
                out.insert(key, (*m).to_string());
            }
            // Менял только игрок.
            (Some(m), b, t) if b == t => {
                out.insert(key, (*m).to_string());
            }
            // Менял только сервер.
            (m, b, Some(t)) if m == b => {
                out.insert(key, (*t).to_string());
            }
            // Ключ появился только у одной стороны.
            (Some(m), None, None) => {
                out.insert(key, (*m).to_string());
            }
            (None, None, Some(t)) => {
                out.insert(key, (*t).to_string());
            }
            // Оба изменили один ключ по-разному — здесь автоматика кончается.
            _ => return None,
        }
    }

    Some(
        out.into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// `key=value` построчно. Комментарии и пустые строки пропускаются: сохранять
/// их порядок при слиянии всё равно нечем.
fn parse(text: &str) -> BTreeMap<&str, &str> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('!'))
        .filter_map(|l| l.split_once(&['=', ':'][..]))
        .map(|(k, v)| (k.trim(), v.trim()))
        .collect()
}

/// Поддерживается ли формат.
pub fn is_mergeable(rel_path: &str) -> bool {
    let lower = rel_path.to_lowercase();
    lower.ends_with(".properties") || lower.ends_with("options.txt")
}

/// Попробовать разрешить конфликт слиянием по ключам.
///
/// Серверную версию приходится скачать до решения — иначе сливать не с чем.
/// Это дёшево ровно потому, что режим включается только для конфигов: они
/// весят килобайты, и лишний GET случается только при настоящем конфликте.
///
/// `None` — не смогли: формат не тот, копии базы нет либо один ключ изменили
/// обе стороны по-разному. Тогда решает политика конфликта.
pub async fn try_merge(
    client: &reqwest::Client,
    instance_dir: &Path,
    rel: &str,
    url: &str,
) -> Option<String> {
    if !is_mergeable(rel) {
        return None;
    }
    let base = tokio::fs::read_to_string(base_copy_path(instance_dir, rel))
        .await
        .ok()?;
    let mine = tokio::fs::read_to_string(instance_dir.join(rel))
        .await
        .ok()?;
    let theirs = client.get(url).send().await.ok()?.text().await.ok()?;

    let merged = merge_properties(&mine, &base, &theirs)?;
    tokio::fs::write(instance_dir.join(rel), &merged)
        .await
        .ok()?;
    // Новая база — то, что сейчас у сервера: следующий раз сравниваем с ним.
    let base_path = base_copy_path(instance_dir, rel);
    if let Some(parent) = base_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let _ = tokio::fs::write(base_path, &theirs).await;
    Some(merged)
}

#[cfg(test)]
#[path = "../keymerge_tests.rs"]
mod tests;
