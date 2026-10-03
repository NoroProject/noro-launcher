//! Copies of the configs as the server last installed them.
//!
//! Key-level merging needs the actual text of the base version, not just its
//! hash the way the file-level three-way does.

use super::is_mergeable;
use std::path::Path;

pub fn base_copy_path(instance_dir: &Path, rel: &str) -> std::path::PathBuf {
    instance_dir.join(".noro/base").join(rel)
}

/// Only copies paths we can actually merge by key — keeping a copy of every
/// config for a format we can't merge costs disk and buys nothing.
///
/// Call it only for a file that was just downloaded, when what sits on disk is
/// the server's text. After a kept or merged conflict the disk holds the
/// player's version, and recording that as the base makes the next merge
/// believe the player never changed anything — so it takes every value from
/// the server and wipes their edits.
pub async fn remember_base(instance_dir: &Path, rel: &str) {
    if !is_mergeable(rel) {
        return;
    }
    let Ok(text) = tokio::fs::read(instance_dir.join(rel)).await else {
        return;
    };
    let _ = crate::fsutil::write_atomic(base_copy_path(instance_dir, rel), text).await;
}
