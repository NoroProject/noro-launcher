//! `merged` mode: a three-way compare by hash, without keeping file copies.
//!
//! Under `user_managed` a file is never updated again — the server fixes a mod
//! config and the player stays on the old one forever, even having never
//! touched it. All this needs to do better is one json: `.noro/base-hashes.json`
//! holds the sha1 of whatever we installed last time, and comparing mine and
//! theirs against that base says who actually changed the file.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

const BASE_PATH: &str = ".noro/base-hashes.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Untouched by the player — take the server's version.
    Update,
    /// Unchanged on the server — keep the player's edits.
    KeepMine,
    /// Both sides changed it.
    Conflict,
    Nothing,
}

/// `mine` is `None` when the file is missing — install it, whoever changed
/// what. `base` is `None` on the first pass, which also means install: there is
/// no base to argue with yet.
pub fn decide(mine: Option<&str>, base: Option<&str>, theirs: &str) -> Decision {
    let Some(mine) = mine else {
        return Decision::Update;
    };
    let Some(base) = base else {
        // No base to compare against. Matching the server means nothing to do;
        // differing means edits we never saw, and those are worth a conflict
        // rather than an overwrite.
        return if mine == theirs {
            Decision::Nothing
        } else {
            Decision::Conflict
        };
    };

    match (mine == base, theirs == base) {
        (true, false) => Decision::Update,
        (false, true) => Decision::KeepMine,
        (false, false) => {
            if mine == theirs {
                Decision::Nothing
            } else {
                Decision::Conflict
            }
        }
        (true, true) => Decision::Nothing,
    }
}

/// Hashes of what we installed last time.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BaseHashes(HashMap<String, String>);

impl BaseHashes {
    pub async fn load(instance_dir: &Path) -> Self {
        match tokio::fs::read(instance_dir.join(BASE_PATH)).await {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn get(&self, path: &str) -> Option<&str> {
        self.0.get(path).map(String::as_str)
    }

    pub fn set(&mut self, path: &str, sha1: &str) {
        self.0.insert(path.to_string(), sha1.to_string());
    }

    /// A failed write doesn't abort the launch: without a base the next pass
    /// treats the files as unknown and refuses to overwrite anything.
    pub async fn save(&self, instance_dir: &Path) {
        let path = instance_dir.join(BASE_PATH);
        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        if let Ok(bytes) = serde_json::to_vec(&self.0) {
            let _ = tokio::fs::write(path, bytes).await;
        }
    }
}

/// Set the player's version aside before taking the server's. Without the copy,
/// resolving a conflict would silently destroy their edits.
pub async fn backup_conflict(instance_dir: &Path, rel: &str, stamp: &str) -> std::io::Result<()> {
    let src = instance_dir.join(rel);
    let dst = instance_dir.join(".noro/conflicts").join(stamp).join(rel);
    if let Some(parent) = dst.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::copy(&src, &dst).await?;
    Ok(())
}

#[cfg(test)]
#[path = "merge_tests.rs"]
mod tests;
