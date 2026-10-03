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
    /// A missing file gives the default. A file that exists but doesn't parse
    /// is moved aside as `<name>.broken` before the default takes over, so the
    /// first `save` can't destroy the only copy of the player's settings, and
    /// the reason ends up in the log rather than nowhere.
    pub fn load(path: PathBuf) -> Self {
        let value = match std::fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(value) => value,
                Err(e) => {
                    let mut aside = path.clone().into_os_string();
                    aside.push(".broken");
                    tracing::error!(
                        path = %path.display(),
                        error = %format!("{e:#}"),
                        "settings file did not parse, starting from defaults; the old file is kept as .broken"
                    );
                    let _ = std::fs::rename(&path, aside);
                    T::default()
                }
            },
            Err(_) => T::default(),
        };
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

    /// Atomic: an interrupted save leaves the previous file, not an empty one.
    pub fn save(&self) {
        let json = match serde_json::to_string_pretty(&*self.value.read()) {
            Ok(json) => json,
            Err(e) => {
                tracing::error!(error = %format!("{e:#}"), "settings did not serialize");
                return;
            }
        };
        if let Err(e) = crate::fsutil::write_atomic_sync(&self.path, json.as_bytes()) {
            tracing::error!(path = %self.path.display(), error = %format!("{e:#}"), "settings not saved");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_http::TempDir;

    #[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Sample {
        value: u32,
    }

    #[test]
    fn a_broken_file_is_kept_aside_instead_of_overwritten() {
        let dir = TempDir::new("persistent");
        let path = dir.path().join("config.json");
        std::fs::write(&path, "{\"value\": 7").unwrap();

        let loaded = Persistent::<Sample>::load(path.clone());
        assert_eq!(loaded.get(), Sample::default());
        loaded.update(|s| s.value = 1);

        assert_eq!(
            std::fs::read_to_string(dir.path().join("config.json.broken")).unwrap(),
            "{\"value\": 7"
        );
        let reread = Persistent::<Sample>::load(path);
        assert_eq!(reread.get().value, 1);
    }
}
