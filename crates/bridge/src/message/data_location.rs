//! The game files folder and moving it, as the window sees them.

use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLocationView {
    pub path: PathBuf,
    /// The launcher's own folder, where a fresh install keeps everything.
    pub default_path: PathBuf,
    pub is_default: bool,
    pub status: DataStatus,
    /// `None` until counted: walking a modpack's files takes a moment, and is
    /// only done when the settings page asks.
    pub size_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DataStatus {
    #[default]
    Ready,
    /// The folder is gone — most likely an external disk that is not plugged
    /// in. Nothing downloads until the player reconnects it or goes back to
    /// the standard folder.
    Unavailable,
    /// A move stopped half way, by a crash or a closed launcher. The game
    /// still uses `from`; the player picks whether to finish or undo it.
    Interrupted { from: PathBuf, to: PathBuf },
}

/// What a move will do, for the player to confirm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataMovePlan {
    /// Where the files will end up: the folder picked, or a new folder inside
    /// it when the one picked already holds other things.
    pub target: PathBuf,
    pub bytes: u64,
    /// `None` where the system wouldn't say.
    pub free: Option<u64>,
    /// A rename: instant, and needs no room.
    pub same_volume: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataMovePhase {
    Copying,
    Verifying,
    Switching,
    Cleaning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataMoveProgress {
    pub phase: DataMovePhase,
    pub done_bytes: u64,
    pub total_bytes: u64,
    pub done_files: u64,
    pub total_files: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataMoveOutcome {
    /// `leftover` is what could not be deleted from the old place.
    Done {
        path: PathBuf,
        leftover: Option<PathBuf>,
    },
    Cancelled,
    /// Nothing changed; the game still uses `path`.
    Failed {
        path: PathBuf,
        key: String,
        args: BTreeMap<String, String>,
    },
}
