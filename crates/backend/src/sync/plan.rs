//! What to do with each file in the manifest, driven by the path rules.

use super::merge::{self, BaseHashes, Decision};
use crate::directories::safe_join;
use schema::{BuildManifest, ConflictPolicy, FileEntry, PathMode};
use std::path::Path;

pub enum Action {
    Download,
    Skip,
    /// Both sides changed the file.
    Conflict(ConflictPolicy),
}

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

        // Installed once, then it belongs to the player. It never gets an
        // update again, which is what `Merged` exists to fix.
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
