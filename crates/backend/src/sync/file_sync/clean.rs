//! Удаление лишнего, защита путей от него и отбор активных опциональных модов.

use anyhow::Result;
use schema::{BuildManifest, FileEntry, UserProfile};
use std::collections::HashSet;
use std::path::Path;

/// Пути файлов выключенных (или недоступных по правам) опциональных модов.
pub fn excluded_optional_files(
    manifest: &BuildManifest,
    enabled: &[String],
    user: &UserProfile,
) -> HashSet<String> {
    let mut excluded = HashSet::new();
    for m in &manifest.optional_mods {
        let user_enabled = enabled.iter().any(|n| n == &m.name);
        let allowed = user.can_use_optional(&manifest.server_id, &m.name, m.limited);
        let active = if m.limited {
            user_enabled && allowed
        } else if enabled.is_empty() {
            // нелимитный без выбора игрока: по умолчанию
            m.enabled_by_default
        } else {
            user_enabled
        };
        if !active {
            for f in &m.files {
                excluded.insert(f.clone());
            }
        }
    }
    excluded
}

/// Удалить файлы, отсутствующие в effective и не попадающие под защищённые пути.
pub(super) async fn clean_extra(
    instance_dir: &Path,
    effective: &[&FileEntry],
    manifest: &BuildManifest,
) -> Result<()> {
    let keep: HashSet<String> = effective.iter().map(|f| f.path.clone()).collect();
    // Всё, что не принадлежит сборке целиком, из удаления исключено: правила
    // разрешают удалять только managed-пути.
    let rules = manifest.path_rules.clone();
    // Служебные внутренние пути лаунчера. `.noro/` — база хешей и отложенные
    // конфликты: снести их значит потерять и то, и другое.
    let protected: Vec<String> = vec![
        ".natives/".to_string(),
        ".noro/".to_string(),
        ".noro-build".to_string(),
        ".noro-servers".to_string(),
    ];

    let root = instance_dir.to_path_buf();
    let to_delete = tokio::task::spawn_blocking(move || {
        let mut victims = Vec::new();
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = match entry.path().strip_prefix(&root) {
                Ok(r) => r.to_string_lossy().replace('\\', "/"),
                Err(_) => continue,
            };
            if keep.contains(&rel) {
                continue;
            }
            if is_protected(&rel, &protected) {
                continue;
            }
            if schema::mode_for(&rel, &rules) != schema::PathMode::Managed {
                continue;
            }
            victims.push(entry.path().to_path_buf());
        }
        victims
    })
    .await?;

    for path in to_delete {
        let _ = tokio::fs::remove_file(&path).await;
    }
    Ok(())
}

/// Защищён ли относительный путь одним из префиксов из манифеста мастера (директория с '/', точный путь или маска '*').
pub fn is_protected(rel: &str, protected: &[String]) -> bool {
    let rel_lower = rel.to_lowercase();
    protected
        .iter()
        .any(|p| match_path_pattern(&rel_lower, &p.to_lowercase()))
}

fn match_path_pattern(rel: &str, pattern: &str) -> bool {
    if pattern.ends_with('*') {
        let prefix = pattern.trim_end_matches('*');
        rel.starts_with(prefix)
    } else if let Some(dir) = pattern.strip_suffix('/') {
        rel == dir || rel.starts_with(&format!("{dir}/"))
    } else {
        rel == pattern
    }
}

#[cfg(test)]
#[path = "clean_tests.rs"]
mod tests;
