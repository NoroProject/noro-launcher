//! The per-file decision, and what to do when both sides changed a file.

use super::ProgressFn;
use crate::directories::safe_join;
use crate::sync::downloader::DownloadTask;
use crate::sync::merge::BaseHashes;
use crate::sync::plan;
use anyhow::{bail, Result};
use bridge::SyncStage;
use schema::{ArtifactKind, BuildManifest, FileEntry};
use std::path::Path;
use std::sync::Arc;

/// The download list, plus the base hashes the next pass will compare against.
#[allow(clippy::too_many_arguments)]
pub(super) async fn collect(
    client: &reqwest::Client,
    instance_dir: &Path,
    manifest: &BuildManifest,
    effective: &[&FileEntry],
    mut base: BaseHashes,
    stamp: &str,
    progress: &ProgressFn,
    cancelled: &Arc<dyn Fn() -> bool + Send + Sync>,
) -> Result<(Vec<(ArtifactKind, DownloadTask)>, BaseHashes)> {
    progress(
        SyncStage::CheckingFiles,
        0,
        effective.len() as u64,
        String::new(),
    );
    let mut tasks: Vec<(ArtifactKind, DownloadTask)> = Vec::new();
    for (i, f) in effective.iter().enumerate() {
        if cancelled() {
            bail!("cancelled");
        }
        let Some(dest) = safe_join(instance_dir, &f.path) else {
            continue;
        };
        let kind = manifest.kind_of(&f.path);
        let verify_hash = matches!(
            kind,
            ArtifactKind::Mod
                | ArtifactKind::Config
                | ArtifactKind::ClientJar
                | ArtifactKind::Other
        );
        let wanted = match plan::decide_file(instance_dir, manifest, f, &base, verify_hash).await {
            plan::Action::Download => true,
            plan::Action::Skip => false,
            // Try a key merge before involving a human: edits to different
            // lines of the same config aren't really in conflict.
            plan::Action::Conflict(_)
                if crate::sync::keymerge::try_merge(client, instance_dir, &f.path, &f.url)
                    .await
                    .is_some() =>
            {
                tracing::info!(path = %f.path, "conflict resolved by key merge");
                false
            }
            plan::Action::Conflict(policy) => match policy {
                schema::ConflictPolicy::KeepMine => {
                    tracing::warn!(path = %f.path, "conflict: kept the player's version");
                    false
                }
                // Take the server's, but set the player's copy aside first.
                schema::ConflictPolicy::TakeTheirs => {
                    if let Err(e) =
                        crate::sync::merge::backup_conflict(instance_dir, &f.path, stamp).await
                    {
                        tracing::warn!(path = %f.path, error = %e, "could not back up the player's version");
                    }
                    true
                }
            },
        };
        // Record what the server is serving now; the next pass compares against
        // it to work out which side changed the file.
        if wanted {
            base.set(&f.path, &f.sha1);
        }
        if wanted {
            tasks.push((
                kind,
                DownloadTask {
                    url: f.url.clone(),
                    dest,
                    sha1: f.sha1.clone(),
                    size: f.size,
                    executable: f.executable,
                },
            ));
        }
        if i % 64 == 0 {
            progress(
                SyncStage::CheckingFiles,
                i as u64,
                effective.len() as u64,
                String::new(),
            );
        }
    }

    Ok((tasks, base))
}
