//! Что именно сверяется и что считается лишним.

use super::finding;
use crate::directories::safe_join;
use crate::sync::file_sync::is_protected;
use schema::{
    ArtifactKind, BuildManifest, FileEntry, IntegrityFinding, IntegrityKind, UserProfile,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Виды файлов, которые вообще имеет смысл сверять.
///
/// Ассеты, библиотеки и JRE — это десятки тысяч файлов на гигабайты; их подмена
/// не даёт игроку ничего, ради чего стоило бы держать его перед пустым окном
/// лишние полминуты. Смысл проверки в модах и конфигах.
fn is_checked(kind: ArtifactKind) -> bool {
    matches!(
        kind,
        ArtifactKind::Mod | ArtifactKind::Config | ArtifactKind::ClientJar
    )
}

/// Файлы манифеста, которые должны лежать на диске именно сейчас.
pub fn expected_files<'a>(
    manifest: &'a BuildManifest,
    enabled_optional: &[String],
    user: &UserProfile,
) -> Vec<&'a FileEntry> {
    let off = crate::sync::file_sync::excluded_optional_files(manifest, enabled_optional, user);
    manifest
        .verified_files
        .iter()
        .filter(|f| f.side.needed_on_client())
        .filter(|f| f.matches_platform())
        .filter(|f| is_checked(manifest.kind_of(&f.path)))
        .filter(|f| !off.contains(&f.path))
        // Правки в user-managed — это правки игрока, а не расхождение.
        .filter(|f| !is_protected(&f.path, &manifest.user_managed_paths))
        .filter(|f| !is_protected(&f.path, &manifest.unmanaged_paths))
        .collect()
}

/// Удалить из managed-каталогов то, чего нет в манифесте.
pub async fn remove_extras(
    instance_dir: &Path,
    manifest: &BuildManifest,
    expected: &[&FileEntry],
) -> Vec<IntegrityFinding> {
    // Ищем только там, где файлы принадлежат сборке целиком. Обход всего
    // инстанса — это `saves/` на гигабайты и десятки тысяч ассетов.
    const MANAGED_DIRS: [&str; 2] = ["mods", "config"];

    let known: HashSet<&str> = expected.iter().map(|f| f.path.as_str()).collect();
    // Выключенный опциональный мод — не лишний файл: он лежит на месте и ждёт,
    // когда его включат обратно.
    let from_manifest: HashSet<&str> = manifest
        .verified_files
        .iter()
        .map(|f| f.path.as_str())
        .collect();

    let mut protected: Vec<String> = manifest.unmanaged_paths.clone();
    protected.extend(manifest.user_managed_paths.iter().cloned());
    protected.push(".noro/".to_string());

    let mut out = Vec::new();
    for dir in MANAGED_DIRS {
        let Some(root) = safe_join(instance_dir, dir) else {
            continue;
        };
        let Ok(mut entries) = tokio::fs::read_dir(&root).await else {
            continue;
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            if !entry
                .file_type()
                .await
                .map(|t| t.is_file())
                .unwrap_or(false)
            {
                continue;
            }
            let rel = format!("{dir}/{}", entry.file_name().to_string_lossy());
            if known.contains(rel.as_str())
                || from_manifest.contains(rel.as_str())
                || is_protected(&rel, &protected)
            {
                continue;
            }
            let repaired = tokio::fs::remove_file(entry.path()).await.is_ok();
            out.push(finding(IntegrityKind::ExtraFile, &rel, None, repaired));
        }
    }
    out
}

/// Включённые limited-моды, права на которые нет.
///
/// Их файлов в манифесте уже нет (мастер фильтрует), так что сам факт включения
/// означает, что список правил клиент.
pub fn forbidden_optionals(
    manifest: &BuildManifest,
    enabled_optional: &[String],
    user: &UserProfile,
) -> Vec<IntegrityFinding> {
    let known: HashMap<&str, bool> = manifest
        .optional_mods
        .iter()
        .map(|m| (m.name.as_str(), m.limited))
        .collect();

    enabled_optional
        .iter()
        .filter(|name| match known.get(name.as_str()) {
            Some(limited) => *limited && !user.can_use_optional(&manifest.server_id, name, true),
            // Имени нет в манифесте — почти всегда это выбор, сохранённый до
            // того, как мод убрали из сборки. Флаг тут был бы ложным, а ничего
            // сверх него такой клиент не получает: файлов мода на диске нет.
            None => false,
        })
        .map(|name| finding(IntegrityKind::ForbiddenOptionalMod, name, None, false))
        .collect()
}
