// Over 150 lines: deciding about a file and resolving a conflict on it read the
// same hashes and the same paths.
//! The per-file decision, and what to do when both sides changed a file.

use super::ProgressFn;
use crate::directories::safe_join;
use crate::sync::downloader::DownloadTask;
use crate::sync::hash_cache::HashCache;
use crate::sync::merge::BaseHashes;
use crate::sync::plan;
use anyhow::{bail, Result};
use bridge::SyncStage;
use futures::stream::{self, StreamExt};
use schema::{ArtifactKind, BuildManifest, FileEntry};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// What one pass decided.
pub(super) struct Collected {
    pub tasks: Vec<(ArtifactKind, DownloadTask)>,
    /// The base hashes the next pass will compare against.
    pub base: BaseHashes,
    /// Paths that get the server's copy this pass. Only these may become the
    /// key-merge base once the downloads are on disk.
    pub fetched: Vec<String>,
}

/// Files checked at once. Most answers come from the hash cache; the misses
/// hash on the blocking pool, so a handful in flight keeps the disk busy
/// without starving everything else.
const CHECK_PARALLELISM: usize = 8;

struct CheckCtx<'a> {
    instance_dir: &'a Path,
    manifest: &'a BuildManifest,
    base: &'a BaseHashes,
    cache: &'a HashCache,
    cancelled: &'a (dyn Fn() -> bool + Send + Sync),
    progress: &'a ProgressFn,
    checked: AtomicU64,
    total: u64,
}

/// One file's verdict, or `None` when it can't be placed or the pass was
/// cancelled.
async fn check<'a>(
    ctx: &CheckCtx<'_>,
    f: &'a FileEntry,
) -> Option<(&'a FileEntry, PathBuf, ArtifactKind, plan::Action)> {
    if (ctx.cancelled)() {
        return None;
    }
    let dest = safe_join(ctx.instance_dir, &f.path)?;
    let kind = ctx.manifest.kind_of(&f.path);
    let verify_hash = matches!(
        kind,
        ArtifactKind::Mod | ArtifactKind::Config | ArtifactKind::ClientJar | ArtifactKind::Other
    );
    let action = plan::decide_file(
        ctx.instance_dir,
        ctx.manifest,
        f,
        ctx.base,
        verify_hash,
        ctx.cache,
    )
    .await;
    let n = ctx.checked.fetch_add(1, Ordering::Relaxed) + 1;
    if n.is_multiple_of(64) {
        (ctx.progress)(SyncStage::CheckingFiles, n, ctx.total, String::new());
    }
    Some((f, dest, kind, action))
}

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
    cache: &HashCache,
) -> Result<Collected> {
    let total = effective.len() as u64;
    progress(SyncStage::CheckingFiles, 0, total, String::new());

    // Deciding is the slow half — it hashes — and each file's answer is
    // independent of the others, so it runs in parallel. Acting on the answers
    // stays sequential below: conflicts touch the base and the backups.
    let ctx = CheckCtx {
        instance_dir,
        manifest,
        base: &base,
        cache,
        cancelled: cancelled.as_ref(),
        progress,
        checked: AtomicU64::new(0),
        total,
    };
    let checks: Vec<_> = effective.iter().map(|f| check(&ctx, f)).collect();
    let decisions: Vec<_> = stream::iter(checks)
        .buffered(CHECK_PARALLELISM)
        .collect()
        .await;
    if cancelled() {
        bail!("cancelled");
    }

    let mut tasks: Vec<(ArtifactKind, DownloadTask)> = Vec::new();
    let mut fetched = Vec::new();
    for (f, dest, kind, action) in decisions.into_iter().flatten() {
        let wanted = match action {
            plan::Action::Download => true,
            plan::Action::Skip => false,
            // Try a key merge before involving a human: edits to different
            // lines of the same config aren't really in conflict.
            plan::Action::Conflict(_)
                if crate::sync::keymerge::try_merge(
                    client,
                    instance_dir,
                    &f.path,
                    &f.url,
                    &f.sha1,
                )
                .await
                .is_some() =>
            {
                tracing::info!(path = %f.path, "conflict resolved by key merge");
                // The server's version is now accounted for. Without moving the
                // base forward, the next pass sees the same conflict and
                // fetches the file again, every launch.
                base.set(&f.path, &f.sha1);
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
                        tracing::warn!(path = %f.path, error = %format!("{e:#}"), "could not back up the player's version");
                    }
                    true
                }
            },
        };
        // Record what the server is serving now; the next pass compares against
        // it to work out which side changed the file.
        if wanted {
            base.set(&f.path, &f.sha1);
            fetched.push(f.path.clone());
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
    }

    Ok(Collected {
        tasks,
        base,
        fetched,
    })
}
