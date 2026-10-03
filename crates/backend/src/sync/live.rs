// File exceeds 150 lines: live sync comparing hashes, in-place atomic replace, and auto-enabling packs in options.txt.
//! Syncing while the game is running.
//!
//! The normal sync touches the whole instance directory and only runs before
//! launch — under a running game it would delete files the JVM holds open. This
//! one covers just the directories the game reads on demand.
//!
//! Everything else is left alone for a reason: `saves/` has an open
//! `session.lock`, `options.txt` gets rewritten when the game exits, and
//! `mods/` and `config/` are read once at startup, so swapping them does
//! nothing until the next launch anyway.

use crate::directories::safe_join;
use anyhow::Result;
use schema::build::{BuildManifest, FileEntry};
use schema::{PathMode, UserProfile};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const LIVE_DIRS: [&str; 2] = ["resourcepacks/", "shaderpacks/"];

/// Packs the launcher has switched on in `options.txt` before. A pack is
/// switched on once, when it first arrives; after that turning it off is the
/// player's call, and the next launch must not undo it.
const ENABLED_PACKS_PATH: &str = ".noro/enabled-packs.json";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Applied {
    pub updated: Vec<String>,
    /// Held open by the game; will be replaced on the next launch.
    pub locked: Vec<String>,
}

impl Applied {
    pub fn nothing(&self) -> bool {
        self.updated.is_empty() && self.locked.is_empty()
    }
}

pub fn live(path: &str) -> bool {
    LIVE_DIRS.iter().any(|dir| path.starts_with(dir))
}

/// The packs that differ from the manifest. Only files the full sync would
/// install for this player are considered — the same optional-mod, side,
/// platform and path-rule filters — so live sync can never put something into
/// the instance that the next full sync would delete or refuse to touch.
///
/// Compares by hash rather than mtime — a pack may have been dropped in by
/// hand, and its timestamp would look newer than ours.
pub async fn outdated(
    instance_dir: &Path,
    manifest: &BuildManifest,
    enabled_optional: &[String],
    user: &UserProfile,
) -> Vec<FileEntry> {
    let personal: BTreeSet<&str> = manifest
        .personal_content
        .iter()
        .map(|c| c.path.as_str())
        .collect();
    let mut out = Vec::new();
    for entry in crate::sync::file_sync::effective_files(manifest, enabled_optional, user) {
        if !live(&entry.path) {
            continue;
        }
        // `live` only matches a prefix, so `resourcepacks/../../x` passes it.
        let Some(path) = safe_join(instance_dir, &entry.path) else {
            continue;
        };
        let mode = if personal.contains(entry.path.as_str()) {
            PathMode::Managed
        } else {
            schema::mode_for(&entry.path, &manifest.path_rules)
        };
        let stale = match mode {
            PathMode::Managed => !matches(&path, &entry.sha1).await,
            // Installed once, then the player's: only a missing file is ours.
            PathMode::UserManaged => tokio::fs::metadata(&path).await.is_err(),
            // Packs are binary; there is nothing to merge, and a merged path
            // is left to the full sync before launch.
            PathMode::Merged | PathMode::Unmanaged => false,
        };
        if stale {
            out.push(entry.clone());
        }
    }
    out
}

async fn matches(path: &Path, sha1: &str) -> bool {
    crate::sync::integrity::sha1_file(path)
        .await
        .is_ok_and(|had| had.eq_ignore_ascii_case(sha1))
}

/// Where a pack is downloaded before it is swapped in.
fn staging_path(dest: &Path) -> PathBuf {
    let mut p = dest.as_os_str().to_owned();
    p.push(".noro-live");
    PathBuf::from(p)
}

/// Move a fully downloaded and verified file over the real one, unless the
/// game is holding it. On Windows a zip open in the client can't be replaced —
/// not an error, it just lands on the next launch.
pub async fn swap_in(staged: &Path, dest: &Path) -> Result<bool> {
    match tokio::fs::rename(staged, dest).await {
        Ok(()) => Ok(true),
        Err(_) => {
            let _ = tokio::fs::remove_file(staged).await;
            Ok(false)
        }
    }
}

/// Fetch updated packs and shaders while the game runs. Only what actually
/// differs gets downloaded, and only into an instance that is installed: the
/// manifest also arrives for a server the player merely looked at, and that is
/// no reason to start filling its folder.
pub async fn apply(
    client: &reqwest::Client,
    instance_dir: &Path,
    manifest: &BuildManifest,
    enabled_optional: &[String],
    user: &UserProfile,
) -> Result<Applied> {
    let mut done = Applied::default();
    if !crate::sync::file_sync::version_marker(instance_dir).exists() {
        return Ok(done);
    }
    for entry in outdated(instance_dir, manifest, enabled_optional, user).await {
        let Some(dest) = safe_join(instance_dir, &entry.path) else {
            continue;
        };
        // Streamed to disk and checked against the manifest on the way: a
        // shader pack can be hundreds of megabytes, and a bad download must
        // not replace a good file.
        let staged = staging_path(&dest);
        if let Err(e) =
            crate::sync::fetch::fetch_to_file(client, &entry.url, &staged, &entry.sha1, &|_| {})
                .await
        {
            tracing::warn!(path = %entry.path, error = %format!("{e:#}"), "live sync skipped a file");
            let _ = tokio::fs::remove_file(&staged).await;
            continue;
        }
        if swap_in(&staged, &dest).await? {
            if let Some(pack) = pack_name(&entry.path) {
                let _ = enable_once(instance_dir, pack).await;
            }
            done.updated.push(entry.path);
        } else {
            done.locked.push(entry.path);
        }
    }
    Ok(done)
}

/// `resourcepacks/<name>` or `resourcepacks/<name>/…` → `<name>`: a pack is a
/// zip or a folder, and a folder pack's files are not packs of their own.
pub fn pack_name(path: &str) -> Option<&str> {
    let rest = path.strip_prefix("resourcepacks/")?;
    let name = rest.split('/').next()?;
    (!name.is_empty()).then_some(name)
}

/// Switch on every resource pack the build delivers that the launcher has
/// never switched on before. Runs before each launch.
pub async fn enable_delivered_packs(
    instance_dir: &Path,
    manifest: &BuildManifest,
    enabled_optional: &[String],
    user: &UserProfile,
) {
    let packs: BTreeSet<&str> =
        crate::sync::file_sync::effective_files(manifest, enabled_optional, user)
            .into_iter()
            .filter_map(|f| pack_name(&f.path))
            .collect();
    for pack in packs {
        if let Err(e) = enable_once(instance_dir, pack).await {
            tracing::warn!(pack, error = %e, "could not enable a delivered pack");
        }
    }
}

/// [`enable`], but only the first time the launcher sees this pack.
pub async fn enable_once(instance_dir: &Path, pack: &str) -> Result<bool> {
    let path = instance_dir.join(ENABLED_PACKS_PATH);
    let mut seen: BTreeSet<String> = match tokio::fs::read(&path).await {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => BTreeSet::new(),
    };
    if seen.contains(pack) {
        return Ok(false);
    }
    let changed = enable(instance_dir, pack).await?;
    seen.insert(pack.to_string());
    crate::fsutil::write_atomic(&path, serde_json::to_vec(&seen)?).await?;
    Ok(changed)
}

/// List the pack in `options.txt`.
///
/// Downloading is not enough: the game only loads packs named in
/// `resourcePacks`, so without this one lands in the folder and stays unused.
pub async fn enable(instance_dir: &Path, pack: &str) -> Result<bool> {
    let path = instance_dir.join("options.txt");
    let entry = format!("\"file/{pack}\"");
    let text = match tokio::fs::read_to_string(&path).await {
        Ok(t) => t,
        Err(_) => {
            let initial = format!("resourcePacks:[\"vanilla\",\"mod_resources\",{entry}]\n");
            crate::fsutil::write_atomic(&path, initial).await?;
            return Ok(true);
        }
    };
    if let Some(updated) = add_pack(&text, &entry) {
        crate::fsutil::write_atomic(&path, updated).await?;
        return Ok(true);
    }
    if !text.lines().any(|l| l.starts_with("resourcePacks:")) {
        let mut new_text = text;
        if !new_text.ends_with('\n') {
            new_text.push('\n');
        }
        new_text.push_str(&format!(
            "resourcePacks:[\"vanilla\",\"mod_resources\",{entry}]\n"
        ));
        crate::fsutil::write_atomic(&path, new_text).await?;
        return Ok(true);
    }
    Ok(false)
}

/// Appends the pack to `resourcePacks`. Returns the new text, or `None` when the
/// pack is already listed.
pub(super) fn add_pack(text: &str, entry: &str) -> Option<String> {
    let mut out = Vec::new();
    let mut changed = false;
    for line in text.lines() {
        if let Some(list) = line.strip_prefix("resourcePacks:") {
            if list.contains(entry) {
                return None;
            }
            let inner = list.trim().trim_start_matches('[').trim_end_matches(']');
            let joined = if inner.trim().is_empty() {
                entry.to_string()
            } else {
                format!("{inner},{entry}")
            };
            out.push(format!("resourcePacks:[{joined}]"));
            changed = true;
            continue;
        }
        out.push(line.to_string());
    }
    changed.then(|| out.join("\n") + "\n")
}
