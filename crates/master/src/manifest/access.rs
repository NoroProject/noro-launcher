//! Фильтрация манифеста по правам игрока и его системе.
//!
//! Проверка права на limited-мод в клиенте ничего не защищает: репозиторий
//! открыт, свой билд включит что угодно. Единственное, чего пропатченный клиент
//! обойти не может, — отсутствие файла на диске. Поэтому мод, на который нет
//! права, вырезается вместе со своими файлами ещё до подписи манифеста.
//!
//! По той же причине здесь отсеиваются моды для чужой системы. Отдать их и
//! понадеяться на лаунчер нельзя: файлы попали бы в `verified_files`, лаунчер
//! честно скачал бы их, а сверка целостности принялась бы удалять как лишние —
//! и так по кругу, на каждом запуске.

use schema::{BuildManifest, UserProfile};

use std::collections::HashSet;

/// Убрать limited-моды без права и их файлы.
///
/// Побочный эффект: при выдаче права игроку нужен ре-синк для докачки, при
/// отзыве `clean_extra` сам удалит файлы. Выдача прав редка — цена приемлема.
/// `os` — система игрока (`windows`, `macos`, `linux`); пусто — не фильтруем.
pub fn filter_for_viewer(manifest: &mut BuildManifest, viewer: &UserProfile, os: &str) {
    let server_id = manifest.server_id;
    let mut denied: HashSet<String> = HashSet::new();
    let mut kept: HashSet<String> = HashSet::new();

    for m in &manifest.optional_mods {
        let allowed = viewer.can_use_optional(&server_id, &m.name, m.limited) && suits(m, os);
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
        .retain(|m| viewer.can_use_optional(&server_id, &m.name, m.limited) && suits(m, os));
    manifest
        .verified_files
        .retain(|f| !denied.contains(&f.path));
    manifest.artifact_kinds.retain(|p, _| !denied.contains(p));
}

/// Подходит ли мод системе игрока.
///
/// Пустая строка приходит от лаунчеров, выпущенных до того, как платформа стала
/// передаваться при входе: спрятать от них моды значило бы сломать работающие
/// сборки, поэтому такому клиенту отдаём всё.
fn suits(m: &schema::OptionalMod, os: &str) -> bool {
    os.is_empty() || m.runs_on(os)
}

/// `macos-aarch64` → `macos`. Платформу лаунчер шлёт вместе с архитектурой, а
/// моды различаются только системой: нативная библиотека собрана под обе.
pub fn os_of(platform: &str) -> &str {
    platform.split('-').next().unwrap_or("")
}

#[cfg(test)]
#[path = "access_tests.rs"]
mod tests;
