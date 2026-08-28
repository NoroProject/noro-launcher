//! A value kept in sync with a JSON file on disk.

use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct Persistent<T> {
    path: PathBuf,
    value: Arc<RwLock<T>>,
}

impl<T: Serialize + DeserializeOwned + Default + Clone> Persistent<T> {
    /// A missing or unreadable file gives the default — nothing is reported,
    /// and the first `save` overwrites whatever was there.
    pub fn load(path: PathBuf) -> Self {
        let value = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self {
            path,
            value: Arc::new(RwLock::new(value)),
        }
    }

    pub fn get(&self) -> T {
        self.value.read().clone()
    }

    /// Writes through to disk before returning.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        {
            let mut guard = self.value.write();
            f(&mut guard);
        }
        self.save();
    }

    pub fn save(&self) {
        let snapshot = self.value.read();
        if let Ok(json) = serde_json::to_string_pretty(&*snapshot) {
            if let Some(parent) = self.path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&self.path, json);
        }
    }
}
