//! Hash cache keyed on mtime + size.
//!
//! Without it the pre-launch check rehashes every mod and config, which is
//! hundreds of megabytes and a visible pause before the game window. A file
//! whose size and mtime both held still almost certainly didn't change; the
//! "almost" is fine here because this feeds telemetry, not anti-cheat.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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
    entries: HashMap<String, Entry>,
    dirty: bool,
}

impl HashCache {
    pub async fn load(instance_dir: &Path) -> Self {
        let path = instance_dir.join(CACHE_PATH);
        let entries = match tokio::fs::read(&path).await {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => HashMap::new(),
        };
        HashCache {
            entries,
            dirty: false,
        }
    }

    /// `None` when the file is missing or unreadable.
    pub async fn sha1_of(&mut self, path: &Path) -> Option<String> {
        let meta = tokio::fs::metadata(path).await.ok()?;
        let size = meta.len();
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or_default();

        let key = path.to_string_lossy().replace('\\', "/");
        if let Some(hit) = self.entries.get(&key) {
            if hit.size == size && hit.mtime == mtime {
                return Some(hit.sha1.clone());
            }
        }

        let sha1 = super::super::integrity::sha1_file(path).await.ok()?;
        self.entries.insert(
            key,
            Entry {
                size,
                mtime,
                sha1: sha1.clone(),
            },
        );
        self.dirty = true;
        Some(sha1)
    }

    /// A failed write doesn't matter — the next pass rebuilds the cache, and
    /// there's no reason to fail a launch over it.
    pub async fn save(&self, instance_dir: &Path) {
        if !self.dirty {
            return;
        }
        let path = instance_dir.join(CACHE_PATH);
        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }
        if let Ok(bytes) = serde_json::to_vec(&self.entries) {
            let _ = tokio::fs::write(&path, bytes).await;
        }
    }
}
