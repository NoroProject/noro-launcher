//! Импорт zip с корнем сборки: внутри лежат `mods/`, `config/` и прочее.
//!
//! В отличие от mrpack тут нет манифеста и ничего не докачивается по сети —
//! в архиве уже всё, что нужно. Это путь для сборок, собранных вручную.

use crate::state::AppState;
use anyhow::{anyhow, Result};
use std::io::Read;
use uuid::Uuid;

/// Папки, по которым узнаётся корень инстанса.
const INSTANCE_DIRS: &[&str] = &[
    "mods",
    "config",
    "defaultconfigs",
    "resourcepacks",
    "shaderpacks",
    "kubejs",
    "scripts",
    "saves",
    "emotes",
];

/// Мусор архиваторов: в сборке ему делать нечего.
fn is_junk(path: &str) -> bool {
    path.starts_with("__MACOSX/")
        || path.split('/').any(|part| {
            part == ".DS_Store" || part == "Thumbs.db" || part == ".git" || part.is_empty()
        })
}

/// Путь внутри архива → путь внутри сборки, либо `None`, если он опасен.
///
/// Лаунчер разложит эти пути по диску у игрока, поэтому выход за корень
/// сборки надо отсекать здесь, а не надеяться на клиента.
fn safe_path(raw: &str) -> Option<String> {
    let normalized = raw.replace('\\', "/");
    if normalized.starts_with('/') || normalized.contains(':') {
        return None;
    }
    if normalized.split('/').any(|part| part == "..") {
        return None;
    }
    let trimmed = normalized.trim_start_matches("./").to_string();
    (!trimmed.is_empty()).then_some(trimmed)
}

/// Если архив — это заархивированная папка сборки, отдать её префикс.
///
/// Различить «запаковали папку» и «запаковали содержимое» можно только по
/// тому, что лежит внутри: `MyPack/mods/…` надо развернуть, а `mods/…` —
/// нет, иначе от сборки останутся голые файлы модов.
fn wrapper_prefix(paths: &[String]) -> Option<String> {
    let mut top: Option<&str> = None;
    for path in paths {
        let (head, rest) = path.split_once('/')?;
        if rest.is_empty() {
            return None;
        }
        match top {
            Some(known) if known != head => return None,
            Some(_) => {}
            None => top = Some(head),
        }
    }
    let top = top?;
    // Внутри обёртки должна найтись хотя бы одна папка инстанса — иначе это
    // не обёртка, а сама `mods/` или `config/`.
    let prefix = format!("{top}/");
    let has_instance_dir = paths.iter().any(|p| {
        p.strip_prefix(&prefix)
            .and_then(|rest| rest.split_once('/'))
            .is_some_and(|(dir, _)| INSTANCE_DIRS.contains(&dir))
    });
    has_instance_dir.then_some(prefix)
}

pub async fn import(
    state: &AppState,
    build_id: Uuid,
    job_id: Uuid,
    bytes: Vec<u8>,
) -> Result<usize> {
    let cursor = std::io::Cursor::new(&bytes);
    let mut zip = zip::ZipArchive::new(cursor)?;

    // Первый проход — только имена: нужно понять, обёрнут ли архив папкой,
    // прежде чем раскладывать файлы.
    let mut entries: Vec<(usize, String)> = Vec::new();
    for i in 0..zip.len() {
        let entry = zip.by_index(i)?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        if is_junk(&name) {
            continue;
        }
        if let Some(path) = safe_path(&name) {
            entries.push((i, path));
        }
    }
    if entries.is_empty() {
        return Err(anyhow!("в архиве нет файлов"));
    }

    let paths: Vec<String> = entries.iter().map(|(_, p)| p.clone()).collect();
    let prefix = wrapper_prefix(&paths);

    if let Some(mut prog) = state.import_jobs.get_mut(&job_id) {
        prog.total = entries.len();
    }

    let mut count = 0usize;
    for (current, (index, path)) in entries.iter().enumerate() {
        let target = match &prefix {
            Some(p) => path.strip_prefix(p.as_str()).unwrap_or(path).to_string(),
            None => path.clone(),
        };
        if target.is_empty() {
            continue;
        }

        if let Some(mut prog) = state.import_jobs.get_mut(&job_id) {
            prog.current = current;
            prog.current_file = target.clone();
        }

        // Файл читается и сразу уходит в хранилище: держать в памяти всю
        // распакованную сборку нельзя, моды тянут на гигабайты.
        let data = {
            let mut entry = zip.by_index(*index)?;
            let mut buf = Vec::with_capacity(entry.size().min(16 * 1024 * 1024) as usize);
            entry.read_to_end(&mut buf)?;
            buf
        };

        let stored = state.files.put_bytes(&data).await?;
        crate::db::upsert_build_file(
            &state.db,
            build_id,
            &target,
            &stored.sha1,
            stored.size as i64,
            "both",
            super::kind_for(&target),
        )
        .await?;
        count += 1;
    }

    Ok(count)
}

#[cfg(test)]
#[path = "instance_zip_tests.rs"]
mod tests;
