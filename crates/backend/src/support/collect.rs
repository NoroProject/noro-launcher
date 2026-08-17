//! Чтение файлов по allowlist и их очистка.

use super::{Bundle, BundleFile};
use flate2::read::GzDecoder;
use schema::redact;
use schema::BuildManifest;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Потолок на файл. Лог с зациклившимся исключением растёт до гигабайтов, и
/// смысла в его хвосте нет — берём начало и конец.
const MAX_FILE_BYTES: usize = 512 * 1024;

/// Сколько ротированных логов и крешей брать. Больше — это уже архив за месяц,
/// а не разбор конкретной проблемы.
const MAX_ROTATED: usize = 3;
const MAX_CRASHES: usize = 3;

/// Собрать бандл для инстанса.
pub async fn collect(
    instance_dir: &Path,
    manifest: Option<&BuildManifest>,
    enabled_optional: &[String],
) -> Bundle {
    let mut files = Vec::new();

    push(&mut files, instance_dir, "logs/latest.log").await;
    push(&mut files, instance_dir, "logs/debug.log").await;
    push(&mut files, instance_dir, "options.txt").await;

    for rel in newest(instance_dir, "logs", ".log.gz", MAX_ROTATED).await {
        push(&mut files, instance_dir, &rel).await;
    }
    for rel in newest(instance_dir, "crash-reports", ".txt", MAX_CRASHES).await {
        push(&mut files, instance_dir, &rel).await;
    }
    // hs_err JVM кладёт в рабочий каталог процесса — то есть в корень инстанса.
    for rel in newest(instance_dir, "", ".log", MAX_ROTATED).await {
        if rel.starts_with("hs_err_pid") {
            push(&mut files, instance_dir, &rel).await;
        }
    }

    Bundle {
        environment: super::environment::describe(instance_dir, manifest, enabled_optional).await,
        files,
    }
}

/// Прочитать, обрезать, очистить и добавить — если файл есть.
async fn push(out: &mut Vec<BundleFile>, root: &Path, rel: &str) {
    let path = root.join(rel);
    let Ok(meta) = tokio::fs::metadata(&path).await else {
        return;
    };
    let Some(raw) = read_text(&path).await else {
        return;
    };
    out.push(BundleFile {
        name: rel.replace('\\', "/"),
        text: redact(&clamp(&raw)).into_owned(),
        original_bytes: meta.len(),
    });
}

/// Текст файла; `.gz` разжимается — иначе очистить его содержимое нельзя.
async fn read_text(path: &Path) -> Option<String> {
    let bytes = tokio::fs::read(path).await.ok()?;
    if path.extension().is_some_and(|e| e == "gz") {
        let mut text = String::new();
        GzDecoder::new(&bytes[..]).read_to_string(&mut text).ok()?;
        return Some(text);
    }
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

/// Начало и конец: причина обычно в первых строках (что за сборка, что
/// загрузилось), а симптом — в последних.
fn clamp(text: &str) -> String {
    if text.len() <= MAX_FILE_BYTES {
        return text.to_string();
    }
    let half = MAX_FILE_BYTES / 2;
    let head = floor_char_boundary(text, half);
    let tail = ceil_char_boundary(text, text.len() - half);
    format!(
        "{}\n\n[… вырезано {} байт …]\n\n{}",
        &text[..head],
        text.len() - MAX_FILE_BYTES,
        &text[tail..]
    )
}

fn floor_char_boundary(s: &str, mut i: usize) -> usize {
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn ceil_char_boundary(s: &str, mut i: usize) -> usize {
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// Самые свежие файлы с нужным расширением, новые первыми.
async fn newest(root: &Path, subdir: &str, suffix: &str, limit: usize) -> Vec<String> {
    let dir = if subdir.is_empty() {
        root.to_path_buf()
    } else {
        root.join(subdir)
    };
    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return Vec::new();
    };

    let mut found: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    while let Ok(Some(e)) = entries.next_entry().await {
        let name = e.file_name().to_string_lossy().into_owned();
        if !name.ends_with(suffix) {
            continue;
        }
        let Ok(meta) = e.metadata().await else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }
        found.push((meta.modified().unwrap_or(std::time::UNIX_EPOCH), e.path()));
    }
    found.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));

    found
        .into_iter()
        .take(limit)
        .filter_map(|(_, p)| {
            let name = p.file_name()?.to_string_lossy().into_owned();
            Some(if subdir.is_empty() {
                name
            } else {
                format!("{subdir}/{name}")
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "collect_tests.rs"]
mod tests;
