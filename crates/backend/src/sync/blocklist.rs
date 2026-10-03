//! Enforcing the blocked-files list.
//!
//! The walk covers the whole instance rather than only managed paths, because
//! `unmanaged` and `user_managed` are exactly where this has to reach:
//! `resourcepacks/` is never synced, but an xray pack in it still gets removed.

use schema::{BlockAction, BlockedFile, IntegrityFinding, IntegrityKind};
use std::path::Path;

/// Never walked, whatever the rules say — gigabytes of saves and assets that
/// can't hold a blocked file anyway.
const SKIP_DIRS: [&str; 4] = ["saves", "assets", "libraries", "logs"];

#[derive(Default)]
pub struct Report {
    pub findings: Vec<IntegrityFinding>,
    /// A file with the `block_launch` action was found.
    pub block_launch: bool,
}

pub async fn enforce(
    instance_dir: &Path,
    rules: &[BlockedFile],
    cache: &crate::sync::hash_cache::HashCache,
) -> Report {
    let mut report = Report::default();
    if rules.is_empty() {
        return report;
    }
    // The name half of every rule that has one. A file no rule can match by
    // name can't match at all, so it never needs hashing; with only name rules
    // in play, the bundled JRE and the screenshots aren't read every launch.
    // A hash-only rule can match any file, and then everything is hashed.
    let hash_only = rules.iter().any(|r| r.pattern.is_none());
    let by_name: Vec<BlockedFile> = rules
        .iter()
        .filter(|r| r.pattern.is_some())
        .map(|r| BlockedFile {
            sha1: None,
            ..r.clone()
        })
        .collect();

    for (rel, path) in candidates(instance_dir).await {
        if !hash_only && !by_name.iter().any(|r| r.matches(&rel, "")) {
            continue;
        }
        let Some(sha1) = cache.sha1_of(&path).await else {
            continue;
        };
        let Some(rule) = schema::first_match(rules, &rel, &sha1) else {
            continue;
        };

        let repaired = match rule.action {
            BlockAction::Delete => tokio::fs::remove_file(&path).await.is_ok(),
            BlockAction::Flag => false,
            BlockAction::BlockLaunch => {
                report.block_launch = true;
                false
            }
        };
        tracing::warn!(path = %rel, reason = %rule.reason, "blocked file");
        report.findings.push(IntegrityFinding {
            kind: IntegrityKind::ExtraFile,
            subject: rel,
            detail: Some(rule.reason.clone()),
            repaired,
        });
    }
    report
}

async fn candidates(instance_dir: &Path) -> Vec<(String, std::path::PathBuf)> {
    let root = instance_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with(".noro") && !SKIP_DIRS.contains(&name.as_ref())
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                let rel = e
                    .path()
                    .strip_prefix(&root)
                    .ok()?
                    .to_string_lossy()
                    .replace('\\', "/");
                Some((rel, e.path().to_path_buf()))
            })
            .collect()
    })
    .await
    .unwrap_or_default()
}

#[cfg(test)]
#[path = "blocklist_tests.rs"]
mod tests;
