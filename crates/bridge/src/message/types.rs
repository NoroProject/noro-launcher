//! Small types both directions share: sync stages, log lines, build state.

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

/// One line of the game's output, already classified.
#[derive(Debug, Clone)]
pub struct GameLogLine {
    pub timestamp: i64,
    pub level: GameLogLevel,
    pub text: String,
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
