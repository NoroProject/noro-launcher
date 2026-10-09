//! Small types both directions share: sync stages, log lines, build state.

use serde::{Deserialize, Serialize};

/// A step between the finished sync and the game window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchStep {
    /// Checking the instance against the manifest.
    Verifying,
    /// Preparing sign-in and starting the JVM.
    Starting,
}

/// Variant order is also the order of the bars in the UI: download stages run in
/// parallel, and `Ord` keeps the list stable instead of letting it shuffle as
/// progress arrives. Reordering these reorders the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SyncStage {
    CheckingFiles,
    DownloadingJava,
    DownloadingMinecraft,
    DownloadingLibraries,
    DownloadingAssets,
    DownloadingMods,
    ApplyingForgePatches,
    Cleaning,
    Done,
}

impl SyncStage {
    /// Download stages count bytes, the rest count files — the two can't be
    /// added up into a single total.
    pub fn is_download(&self) -> bool {
        matches!(
            self,
            SyncStage::DownloadingJava
                | SyncStage::DownloadingMinecraft
                | SyncStage::DownloadingLibraries
                | SyncStage::DownloadingAssets
                | SyncStage::DownloadingMods
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameLogLevel {
    Info,
    Warn,
    Error,
}

/// How the game console shows the log. Kept in the launcher's config: a console
/// set up once opens the same way next time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConsoleSettings {
    pub show_time: bool,
    pub show_thread: bool,
    pub show_logger: bool,
    /// Long lines wrap rather than run off the edge.
    pub wrap: bool,
    pub font_size: u8,
    pub show_info: bool,
    pub show_warn: bool,
    pub show_error: bool,
}

impl ConsoleSettings {
    pub const FONT_SIZES: [u8; 4] = [11, 12, 13, 14];
}

impl Default for ConsoleSettings {
    fn default() -> Self {
        Self {
            show_time: true,
            // Nearly every line is "Client thread" or "main": column after
            // column of the same word, worth turning on only when it isn't.
            show_thread: false,
            show_logger: true,
            wrap: true,
            font_size: 12,
            show_info: true,
            show_warn: true,
            show_error: true,
        }
    }
}

/// One line of the game's output, taken apart when it arrived.
#[derive(Debug, Clone)]
pub struct GameLogLine {
    pub timestamp: i64,
    pub level: GameLogLevel,
    /// The line as the game printed it: what gets copied and searched.
    pub text: String,
    /// `[Client thread/INFO]` → `Client thread`.
    pub thread: Option<String>,
    /// `[FML]` → `FML`; for a redirected print, the class that printed it.
    pub logger: Option<String>,
    /// The message without the game's own time, thread and logger in front.
    pub body: String,
    /// A frame of a stack trace and the like: the tail of the line above.
    pub continuation: bool,
}

/// An optional mod as the UI sees it, with the player's permissions already
/// resolved.
#[derive(Debug, Clone)]
pub struct OptionalModInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub icon_url: Option<String>,
    pub author: Option<String>,
    /// Gated behind a permission.
    pub limited: bool,
    /// This player has that permission.
    pub allowed: bool,
    pub enabled: bool,
    /// Can't be enabled together with this one.
    pub conflicts: Vec<String>,
    /// This one does nothing without them.
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginErrorKind {
    /// Browser window closed, or the wait ran out.
    Cancelled,
    /// Turned down by Discord or by the master.
    Rejected(String),
    Network(String),
}

/// What the launcher can do with the local copy of a build right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildState {
    #[default]
    Missing,
    Outdated,
    Ready,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerSkinPresetItem {
    pub id: String,
    pub name: String,
    pub skin_url: String,
}
