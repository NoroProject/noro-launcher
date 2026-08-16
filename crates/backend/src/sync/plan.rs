//! Что делать с каждым файлом манифеста — по правилам путей.
//!
//! Вынесено из `file_sync`, потому что решение перестало быть двоичным:
//! раньше был выбор «скачать или не скачать», теперь ещё и three-way для
//! режима `merged`.

use super::merge::{self, BaseHashes, Decision};
use crate::directories::safe_join;
use schema::{BuildManifest, ConflictPolicy, FileEntry, PathMode};
use std::path::Path;

/// Решение по одному файлу.
pub enum Action {
    /// Скачать (поставить или обновить).
    Download,
    /// Оставить как есть.
    Skip,
    /// Конфликт: обе стороны меняли файл.
    Conflict(ConflictPolicy),
}

/// Что делать с файлом.
///
/// `unmanaged` сюда не доходит — такие файлы отсеиваются раньше, до всякой
/// работы с диском.
pub async fn decide_file(
    instance_dir: &Path,
    manifest: &BuildManifest,
    file: &FileEntry,
    base: &BaseHashes,
    verify_hash: bool,
) -> Action {
    let Some(dest) = safe_join(instance_dir, &file.path) else {
        return Action::Skip;
    };
    let rule = schema::rule_for(&file.path, &manifest.path_rules);
    let mode = rule.map(|r| r.mode).unwrap_or_default();

    match mode {
        PathMode::Unmanaged => Action::Skip,

        // Ставится один раз, дальше принадлежит игроку. Обновлений не будет
        // никогда — это и есть дефект, ради которого появился `Merged`.
        PathMode::UserManaged => {
            if dest.exists() {
                Action::Skip
            } else {
                Action::Download
            }
        }

        PathMode::Managed => {
            if super::downloader::needs_download(&dest, file.size, &file.sha1, verify_hash).await {
                Action::Download
            } else {
                Action::Skip
            }
        }

        PathMode::Merged => {
            let mine = match tokio::fs::metadata(&dest).await {
                Ok(_) => super::integrity::sha1_file(&dest).await.ok(),
                Err(_) => None,
            };
            match merge::decide(mine.as_deref(), base.get(&file.path), &file.sha1) {
                Decision::Update => Action::Download,
                Decision::KeepMine | Decision::Nothing => Action::Skip,
                Decision::Conflict => {
                    Action::Conflict(rule.map(|r| r.conflict).unwrap_or_default())
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "plan_tests.rs"]
mod tests;
