//! The pre-launch sync: verify the manifest signature, download what differs,
//! remove what doesn't belong, and honour the optional-mod selection.

use super::downloader::{download_all, DownloadTask};
use anyhow::{bail, Result};
use bridge::SyncStage;
use schema::{BuildManifest, FileEntry, UserProfile};
use std::path::Path;
use std::sync::Arc;

/// Progress callback: (stage, done, total, current file).
pub type ProgressFn = Arc<dyn Fn(SyncStage, u64, u64, String) + Send + Sync>;
mod clean;
mod stages;
mod state;

mod tasks;

use clean::clean_extra;
pub use clean::{excluded_optional_files, is_protected, resolve_enabled};
use stages::STAGE_GROUPS;
pub use state::{build_state, find_java, installed_state, marker_contents, version_marker};

/// The manifest's files this player should have on disk: client-side, for
/// this platform, not belonging to an optional mod that is off, and not under
/// a path the build leaves to the player. Shared by the full sync and the live
/// one, which must never disagree about what belongs to the instance.
pub fn effective_files<'a>(
    manifest: &'a BuildManifest,
    enabled_optional: &[String],
    user: &UserProfile,
) -> Vec<&'a FileEntry> {
    let excluded = excluded_optional_files(manifest, enabled_optional, user);
    // What this player added themselves. Their files skip the `unmanaged`
    // filter below: a build marks `resourcepacks/` unmanaged so it never
    // touches what the player put there by hand, and that rule would otherwise
    // also refuse to install the pack they just asked for through the launcher.
    // Keeping them in `effective` is also what stops `clean_extra` deleting
    // them on the next pass.
    let personal: std::collections::HashSet<&str> = manifest
        .personal_content
        .iter()
        .map(|c| c.path.as_str())
        .collect();
    // Unmanaged paths are dropped here, not merely spared from cleanup: a file
    // still in the download set would land on top of the player's edits.
    manifest
        .verified_files
        .iter()
        .filter(|f| f.side.needed_on_client())
        .filter(|f| !excluded.contains(&f.path))
        .filter(|f| {
            personal.contains(f.path.as_str())
                || schema::mode_for(&f.path, &manifest.path_rules) != schema::PathMode::Unmanaged
        })
        // The build carries the Java runtime and natives for every platform at
        // once. The other platforms' copies are useless and cost several JREs
        // worth of download.
        .filter(|f| f.matches_platform())
        .collect()
}

pub async fn sync_server(
    client: &reqwest::Client,
    instance_dir: &Path,
    manifest: &BuildManifest,
    enabled_optional: &[String],
    user: &UserProfile,
    progress: ProgressFn,
    cancelled: Arc<dyn Fn() -> bool + Send + Sync>,
) -> Result<()> {
    if !crate::signing::verify_manifest(manifest) {
        bail!("manifest signature is invalid, sync aborted");
    }

    tokio::fs::create_dir_all(instance_dir).await?;

    let effective = effective_files(manifest, enabled_optional, user);

    let base = super::merge::BaseHashes::load(instance_dir).await;
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();

    let tasks::Collected {
        tasks,
        base,
        fetched,
    } = tasks::collect(
        client,
        instance_dir,
        manifest,
        &effective,
        base,
        &stamp,
        &progress,
        &cancelled,
    )
    .await?;
    // All stages run at once. They touch disjoint files, and a single big JDK
    // download would otherwise hold up a thousand small assets behind it.
    let jobs = STAGE_GROUPS.iter().filter_map(|g| {
        let group: Vec<DownloadTask> = tasks
            .iter()
            .filter(|(k, _)| g.kinds.contains(k))
            .map(|(_, t)| t.clone())
            .collect();
        if group.is_empty() {
            return None;
        }
        let total: u64 = group.iter().map(|t| t.size).sum();
        // The stage's bar has to be published before the work starts, or it
        // first appears in the UI already half full.
        progress(g.stage, 0, total, String::new());

        let prog = progress.clone();
        let cancelled = cancelled.clone();
        let client = client.clone();
        Some(async move {
            download_all(
                &client,
                group,
                g.concurrency,
                {
                    let prog = prog.clone();
                    move |done| prog(g.stage, done, total, String::new())
                },
                move || cancelled(),
            )
            .await?;
            // The last progress threshold may not have fired; send the exact
            // total so the bar doesn't freeze at 99%.
            prog(g.stage, total, total, String::new());
            Ok::<_, anyhow::Error>(())
        })
    });
    futures::future::try_join_all(jobs).await?;

    // Both of these have to happen after the downloads: before them the
    // server's version isn't on disk yet, and recording its hash would lie to
    // the next pass.
    for rel in &fetched {
        crate::sync::keymerge::remember_base(instance_dir, rel).await;
    }
    base.save(instance_dir).await;

    progress(SyncStage::Cleaning, 0, 0, String::new());
    clean_extra(instance_dir, &effective, manifest).await?;

    // Which build is installed. The files on disk don't say by themselves, so
    // without this there's no telling "install" and "update" from "launch".
    // Without it the button goes back to offering «Install» for a build that
    // is on disk, so a failure here is worth a log line.
    if let Err(e) =
        crate::fsutil::write_atomic(version_marker(instance_dir), marker_contents(manifest)).await
    {
        tracing::warn!(error = %e, "build marker not written");
    }

    progress(SyncStage::Done, 1, 1, String::new());
    Ok(())
}
