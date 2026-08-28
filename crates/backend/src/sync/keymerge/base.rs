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
pub async fn remember_base(instance_dir: &Path, rel: &str) {
    if !is_mergeable(rel) {
        return;
    }
    let dst = base_copy_path(instance_dir, rel);
    if let Some(parent) = dst.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let _ = tokio::fs::copy(instance_dir.join(rel), dst).await;
}
