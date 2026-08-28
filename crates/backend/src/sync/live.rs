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

use anyhow::Result;
use schema::build::{BuildManifest, FileEntry};
use std::path::Path;

const LIVE_DIRS: [&str; 2] = ["resourcepacks/", "shaderpacks/"];

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

/// Compares by hash rather than mtime — a pack may have been dropped in by
/// hand, and its timestamp would look newer than ours.
pub async fn outdated(instance_dir: &Path, manifest: &BuildManifest) -> Vec<FileEntry> {
    let mut out = Vec::new();
    for entry in manifest.verified_files.iter().filter(|f| live(&f.path)) {
        let path = instance_dir.join(&entry.path);
        if !matches(&path, &entry.sha1).await {
            out.push(entry.clone());
        }
    }
    out
}

async fn matches(path: &Path, sha1: &str) -> bool {
    crate::sync::integrity::sha1_file(path)
        .await
        .is_ok_and(|had| had == sha1)
}

/// Replace a file unless the game is holding it. On Windows a zip open in the
/// client can't be replaced — not an error, it just lands on the next launch.
pub async fn replace(path: &Path, bytes: &[u8]) -> Result<bool> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    // Temp file alongside: an interrupted write must not leave half a pack
    // sitting under the real name.
    let tmp = path.with_extension("noro-part");
    tokio::fs::write(&tmp, bytes).await?;
    match tokio::fs::rename(&tmp, path).await {
        Ok(()) => Ok(true),
        Err(_) => {
            let _ = tokio::fs::remove_file(&tmp).await;
            Ok(false)
        }
    }
}

/// Fetch updated packs and shaders while the game runs. Only what actually
/// differs gets downloaded.
pub async fn apply(
    client: &reqwest::Client,
    instance_dir: &Path,
    manifest: &BuildManifest,
) -> Result<Applied> {
    let mut done = Applied::default();
    for entry in outdated(instance_dir, manifest).await {
        let bytes = client.get(&entry.url).send().await?.bytes().await?;
        // The master may serve something other than it promised; skip the file
        // and keep what the player already has.
        if hex::encode(<sha1::Sha1 as sha1::Digest>::digest(&bytes)) != entry.sha1 {
            continue;
        }
        if replace(&instance_dir.join(&entry.path), &bytes).await? {
            if let Some(name) = entry.path.strip_prefix("resourcepacks/") {
                let _ = enable(instance_dir, name).await;
            }
            done.updated.push(entry.path);
        } else {
            done.locked.push(entry.path);
        }
    }
    Ok(done)
}

/// List the pack in `options.txt`.
///
/// Downloading is not enough: the game only loads packs named in
/// `resourcePacks`, so without this one lands in the folder and stays unused.
pub async fn enable(instance_dir: &Path, pack: &str) -> Result<bool> {
    let path = instance_dir.join("options.txt");
    let Ok(text) = tokio::fs::read_to_string(&path).await else {
        // The game hasn't written its settings yet; catch it next time.
        return Ok(false);
    };
    let entry = format!("\"file/{pack}\"");
    let Some(updated) = add_pack(&text, &entry) else {
        return Ok(false);
    };
    tokio::fs::write(&path, updated).await?;
    Ok(true)
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
