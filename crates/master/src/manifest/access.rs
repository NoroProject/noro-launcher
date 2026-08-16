//! Фильтрация манифеста по правам игрока.
//!
//! Проверка права на limited-мод в клиенте ничего не защищает: репозиторий
//! открыт, свой билд включит что угодно. Единственное, чего пропатченный клиент
//! обойти не может, — отсутствие файла на диске. Поэтому мод, на который нет
//! права, вырезается вместе со своими файлами ещё до подписи манифеста.

use schema::{BuildManifest, UserProfile};
use std::collections::HashSet;

/// Убрать limited-моды без права и их файлы.
///
/// Побочный эффект: при выдаче права игроку нужен ре-синк для докачки, при
/// отзыве `clean_extra` сам удалит файлы. Выдача прав редка — цена приемлема.
pub fn filter_for_viewer(manifest: &mut BuildManifest, viewer: &UserProfile) {
    let server_id = manifest.server_id;
    let mut denied: HashSet<String> = HashSet::new();
    let mut kept: HashSet<String> = HashSet::new();

    for m in &manifest.optional_mods {
        let allowed = viewer.can_use_optional(&server_id, &m.name, m.limited);
        let bucket = if allowed { &mut kept } else { &mut denied };
        bucket.extend(m.files.iter().cloned());
    }
    if denied.is_empty() {
        return;
    }

    // Файл, общий с доступным модом, остаётся: иначе отсутствие права на один
    // мод ломало бы другой, разрешённый.
    denied.retain(|p| !kept.contains(p));

    manifest
        .optional_mods
        .retain(|m| viewer.can_use_optional(&server_id, &m.name, m.limited));
    manifest
        .verified_files
        .retain(|f| !denied.contains(&f.path));
    manifest.artifact_kinds.retain(|p, _| !denied.contains(p));
}

#[cfg(test)]
#[path = "access_tests.rs"]
mod tests;
