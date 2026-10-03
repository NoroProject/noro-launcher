//! Hash cache keyed on mtime + size, shared by everything that hashes the
//! instance: the pre-launch sync, the integrity check, the blocklist and the
//! inventory.
//!
//! Without it every launch rehashes every mod and config — hundreds of
//! megabytes, read up to three times — before the game window appears. A file
//! whose size and mtime both held still almost certainly didn't change.
//!
//! Interior mutability so many files can be hashed at once from one cache.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

const CACHE_PATH: &str = ".noro/hash-cache.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Entry {
    size: u64,
    /// Whole unix seconds — filesystems round sub-second precision differently.
    mtime: i64,
    sha1: String,
}

#[derive(Default)]
pub struct HashCache {
    entries: Mutex<HashMap<String, Entry>>,
    dirty: AtomicBool,
}

fn key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

async fn stamp(path: &Path) -> Option<(u64, i64)> {
    let meta = tokio::fs::metadata(path).await.ok()?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default();
    Some((meta.len(), mtime))
}

impl HashCache {
    pub async fn load(instance_dir: &Path) -> Self {
        let entries = match tokio::fs::read(instance_dir.join(CACHE_PATH)).await {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => HashMap::new(),
        };
        HashCache {
            entries: Mutex::new(entries),
            dirty: AtomicBool::new(false),
        }
    }

    /// `None` when the file is missing or unreadable.
    pub async fn sha1_of(&self, path: &Path) -> Option<String> {
        let (size, mtime) = stamp(path).await?;
        let key = key(path);
        if let Some(hit) = self.entries.lock().get(&key) {
            if hit.size == size && hit.mtime == mtime {
                return Some(hit.sha1.clone());
            }
        }
        let sha1 = super::integrity::sha1_file(path).await.ok()?;
        self.store(key, size, mtime, sha1.clone());
        Some(sha1)
    }

    /// A file whose hash is already known — just downloaded and verified on the
    /// way in — goes into the cache without being read again.
    pub async fn record(&self, path: &Path, sha1: &str) {
        if let Some((size, mtime)) = stamp(path).await {
            self.store(key(path), size, mtime, sha1.to_ascii_lowercase());
        }
    }

    fn store(&self, key: String, size: u64, mtime: i64, sha1: String) {
        self.entries.lock().insert(key, Entry { size, mtime, sha1 });
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// A failed write doesn't matter — the next pass rebuilds the cache, and
    /// there's no reason to fail a launch over it.
    pub async fn save(&self, instance_dir: &Path) {
        if !self.dirty.swap(false, Ordering::Relaxed) {
            return;
        }
        let bytes = serde_json::to_vec(&*self.entries.lock());
        if let Ok(bytes) = bytes {
            let _ = crate::fsutil::write_atomic(instance_dir.join(CACHE_PATH), bytes).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_http::{sha1_hex, TempDir};

    #[tokio::test]
    async fn a_recorded_hash_is_served_without_reading_the_file() {
        let dir = TempDir::new("hash-cache");
        let path = dir.path().join("mods/a.jar");
        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .unwrap();
        tokio::fs::write(&path, b"jar").await.unwrap();

        let cache = HashCache::default();
        cache.record(&path, "ABC").await;
        assert_eq!(cache.sha1_of(&path).await.as_deref(), Some("abc"));
    }

    #[tokio::test]
    async fn a_changed_file_is_hashed_again_and_survives_a_reload() {
        let dir = TempDir::new("hash-cache");
        let path = dir.path().join("a.txt");
        tokio::fs::write(&path, b"one").await.unwrap();

        let cache = HashCache::load(dir.path()).await;
        assert_eq!(cache.sha1_of(&path).await, Some(sha1_hex(b"one")));
        tokio::fs::write(&path, b"three").await.unwrap();
        assert_eq!(cache.sha1_of(&path).await, Some(sha1_hex(b"three")));
        cache.save(dir.path()).await;

        let reloaded = HashCache::load(dir.path()).await;
        assert_eq!(reloaded.sha1_of(&path).await, Some(sha1_hex(b"three")));
    }
}
