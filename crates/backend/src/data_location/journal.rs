//! The record of a move in progress, in the service root. Only one launcher
//! runs at a time, so a journal found at start belongs to a move that died.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Journal {
    pub from: PathBuf,
    pub to: PathBuf,
    pub phase: Phase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Across volumes; the config still points at `from`.
    Copying,
    /// Same volume; some entries may already sit in `to`, the config still
    /// points at `from`.
    Renaming,
    /// The config points at `to`; what is left in `from` is to be deleted.
    Cleanup,
}

impl Journal {
    /// An unreadable journal counts as none: there is nothing it could tell
    /// that would be safe to act on.
    pub fn load(path: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&text).ok()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        crate::fsutil::write_atomic_sync(path, &json)
    }

    pub fn clear(path: &Path) {
        let _ = std::fs::remove_file(path);
    }

    /// Where the move really stands, given the folder the config points at.
    /// A crash between writing the config and updating the journal leaves the
    /// journal one phase behind; the config is what decides.
    pub fn settled(mut self, current_data_root: &Path) -> Self {
        if self.phase != Phase::Cleanup && current_data_root == self.to {
            self.phase = Phase::Cleanup;
        }
        self
    }
}
