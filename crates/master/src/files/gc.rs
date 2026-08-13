//! Поиск объектов FileStore, на которые больше никто не ссылается.
//!
//! Хранилище адресуется по содержимому и не удаляет ничего само: снятый со
//! сборки мод остаётся на диске навсегда. Пока сборки заводили руками, это было
//! незаметно; с копированием версий мусор растёт заметно быстрее.
//!
//! **Живое множество собирается сканированием всех текстовых колонок БД**, а не
//! списком таблиц. Список пришлось бы дополнять при каждой новой колонке, и
//! забытая ссылка означала бы удаление живого файла. Здесь ошибка возможна
//! только в безопасную сторону: посторонняя 40-символьная hex-строка (например,
//! git-ревизия) просто оставит лишний объект на диске.

use crate::files::FileStore;
use anyhow::Result;
use sqlx::{PgPool, Row};
use std::collections::HashSet;
use std::time::{Duration, SystemTime};

/// Итог обхода.
#[derive(serde::Serialize, Default)]
pub struct Report {
    pub orphan_count: u64,
    pub orphan_bytes: u64,
    pub deleted: bool,
    /// Несколько первых sha1 — чтобы глазами убедиться, что это мусор.
    pub sample: Vec<String>,
}

/// Объекты моложе этого возраста не трогаются: файл может быть уже загружен, но
/// ещё не привязан к сборке (идёт импорт модпака).
pub const MIN_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);

const SAMPLE_LIMIT: usize = 20;

/// Пройти по хранилищу. При `delete = false` только считает.
pub async fn collect(db: &PgPool, store: &FileStore, delete: bool) -> Result<Report> {
    let mut live = live_sha1s(db).await?;
    live.extend(derived_sha1s(store).await);
    tracing::info!(live = live.len(), "живых ссылок на объекты найдено");

    let root = store.root().to_path_buf();
    let now = SystemTime::now();
    let mut report = Report {
        deleted: delete,
        ..Default::default()
    };

    let mut orphans = Vec::new();
    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !is_sha1(&name) || live.contains(&name) {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        let too_young = meta
            .modified()
            .ok()
            .and_then(|m| now.duration_since(m).ok())
            .is_none_or(|age| age < MIN_AGE);
        if too_young {
            continue;
        }

        report.orphan_count += 1;
        report.orphan_bytes += meta.len();
        if report.sample.len() < SAMPLE_LIMIT {
            report.sample.push(name);
        }
        orphans.push(entry.into_path());
    }

    if delete {
        for path in orphans {
            if let Err(e) = tokio::fs::remove_file(&path).await {
                tracing::warn!(path = %path.display(), error = %e, "не удалось удалить объект");
            }
        }
    }
    Ok(report)
}

fn is_sha1(name: &str) -> bool {
    name.len() == 40
        && name
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// sha1 объектов, которые в БД не записаны вообще.
///
/// `agent_artifact` кладёт jar агента в хранилище и раздаёт его как
/// `/files/{sha1}`, пересчитывая sha1 на каждый запрос из файла в `agents/`.
/// Ссылки на такой объект нет ни в одной колонке — по одной только БД он
/// выглядит мусором, и уборка снесла бы раздачу агентов до следующего запроса.
async fn derived_sha1s(store: &FileStore) -> HashSet<String> {
    let Some(agents) = store.root().parent().map(|d| d.join("agents")) else {
        return HashSet::new();
    };

    let mut out = HashSet::new();
    for entry in walkdir::WalkDir::new(&agents)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        match crate::files::sha1_file(entry.path()).await {
            Ok(sha1) => {
                out.insert(sha1);
            }
            // Не смогли прочитать — считаем, что объект живой: пропустить
            // ссылку здесь дороже, чем оставить лишний файл на диске.
            Err(e) => {
                tracing::warn!(path = %entry.path().display(), error = %e, "агент не прочитан")
            }
        }
    }
    out
}

/// Все sha1, упомянутые хоть в одной текстовой колонке БД.
async fn live_sha1s(db: &PgPool) -> Result<HashSet<String>> {
    let columns = sqlx::query(
        "SELECT table_name, column_name FROM information_schema.columns
         WHERE table_schema = 'public' AND data_type IN ('text', 'character varying')",
    )
    .fetch_all(db)
    .await?;

    let mut live = HashSet::new();
    for row in columns {
        let table: String = row.try_get("table_name")?;
        let column: String = row.try_get("column_name")?;

        // Идентификаторы приходят из information_schema, но всё равно
        // экранируются: конкатенация в SQL без quote_ident — это инъекция.
        let sql = format!(
            r#"SELECT DISTINCT m[1] AS sha1
               FROM {t}, LATERAL regexp_matches({t}.{c}, '[0-9a-f]{{40}}', 'g') AS m"#,
            t = quote_ident(&table),
            c = quote_ident(&column),
        );
        let found = sqlx::query(&sql).fetch_all(db).await?;
        for row in found {
            if let Ok(sha1) = row.try_get::<String, _>("sha1") {
                live.insert(sha1);
            }
        }
    }
    Ok(live)
}

fn quote_ident(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

#[cfg(test)]
#[path = "gc_tests.rs"]
mod tests;
