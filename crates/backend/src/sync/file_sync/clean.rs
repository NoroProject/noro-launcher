//! Removing files that don't belong, the paths that are exempt from it, and
//! working out which optional mods are actually on.

use anyhow::Result;
use schema::{BuildManifest, FileEntry, UserProfile};
use std::collections::HashSet;
use std::path::Path;

/// Files belonging to optional mods that are off, or that the player has no
/// permission for.
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
            // Unrestricted mod, player never chose: fall back to the default.
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

/// Delete everything not in `effective`. Only managed paths are eligible.
pub(super) async fn clean_extra(
    instance_dir: &Path,
    effective: &[&FileEntry],
    manifest: &BuildManifest,
) -> Result<()> {
    let keep: HashSet<String> = effective.iter().map(|f| f.path.clone()).collect();
    let rules = manifest.path_rules.clone();
    // The launcher's own state. `.noro/` holds the base hashes and the saved
    // conflicts — deleting it loses both.
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

/// Patterns come from the master's manifest and take three forms: a directory
/// ending in `/`, an exact path, or a prefix ending in `*`. Matching is
/// case-insensitive.
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
