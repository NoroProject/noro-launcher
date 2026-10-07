//! Content-addressable local store across instances.
//!
//! Immutable artifacts (Java runtime, Mojang assets, libraries, mods) are stored
//! by their SHA1 hash under `store/xx/xx...` and hardlinked into instance folders.
//! This turns re-downloads and multi-server installs into instant local links with
//! zero additional disk space.

use schema::ArtifactKind;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use tokio::sync::Mutex as TokioMutex;

static STORE_LOCKS: LazyLock<Mutex<HashMap<String, Arc<TokioMutex<()>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Exclusive in-flight lock per SHA1 preventing parallel download collisions.
pub fn lock_for_sha1(sha1: &str) -> Arc<TokioMutex<()>> {
    let mut map = STORE_LOCKS.lock().unwrap();
    if map.len() > 1024 {
        map.retain(|_, v| Arc::strong_count(v) > 1);
    }
    map.entry(sha1.to_string())
        .or_insert_with(|| Arc::new(TokioMutex::new(())))
        .clone()
}

pub async fn acquire_sha1_lock(sha1: &str) -> tokio::sync::OwnedMutexGuard<()> {
    lock_for_sha1(sha1).lock_owned().await
}

pub fn store_path(store_root: &Path, sha1: &str) -> Option<PathBuf> {
    if sha1.len() < 4 || !sha1.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some(store_root.join(&sha1[..2]).join(sha1))
}

pub fn is_cacheable(kind: ArtifactKind) -> bool {
    matches!(
        kind,
        ArtifactKind::Java
            | ArtifactKind::ClientJar
            | ArtifactKind::Library
            | ArtifactKind::Runtime
            | ArtifactKind::Native
            | ArtifactKind::Asset
            | ArtifactKind::AssetIndex
            | ArtifactKind::Mod
    )
}

pub async fn has_valid(store_path: &Path, expected_size: u64) -> bool {
    tokio::fs::metadata(store_path)
        .await
        .map(|m| m.len() == expected_size)
        .unwrap_or(false)
}

pub async fn link_or_copy(src: &Path, dst: &Path) -> std::io::Result<()> {
    if dst.exists() {
        let _ = tokio::fs::remove_file(dst).await;
    }
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    match tokio::fs::hard_link(src, dst).await {
        Ok(()) => Ok(()),
        Err(_) => tokio::fs::copy(src, dst).await.map(|_| ()),
    }
}

pub async fn try_link_from_store(
    store_root: &Path,
    sha1: &str,
    expected_size: u64,
    dest: &Path,
    executable: bool,
) -> bool {
    let Some(src) = store_path(store_root, sha1) else {
        return false;
    };
    if !has_valid(&src, expected_size).await {
        return false;
    }
    if link_or_copy(&src, dest).await.is_err() {
        return false;
    }
    #[cfg(unix)]
    if executable {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = tokio::fs::metadata(dest).await {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = tokio::fs::set_permissions(dest, perms).await;
        }
    }
    let _ = executable;
    true
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
