//! Integrity report for the instance directory.
//!
//! A signal, not proof: the launcher is open source and a custom build can send
//! whatever it likes. Findings are grounds for a manual look, never an autoban.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityKind {
    /// File under a managed path that the manifest doesn't list.
    ExtraFile,
    /// Hash doesn't match the manifest.
    ModifiedFile,
    /// Listed in the manifest, not on disk.
    MissingFile,
    /// A limited mod is on without the permission for it.
    ForbiddenOptionalMod,
}

impl IntegrityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            IntegrityKind::ExtraFile => "extra_file",
            IntegrityKind::ModifiedFile => "modified_file",
            IntegrityKind::MissingFile => "missing_file",
            IntegrityKind::ForbiddenOptionalMod => "forbidden_optional_mod",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityFinding {
    pub kind: IntegrityKind,
    /// Path relative to the instance, or a mod name.
    pub subject: String,
    /// What diverged: expected and actual hash, size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// The launcher already dealt with it, e.g. deleted the extra file.
    #[serde(default)]
    pub repaired: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub server_id: Uuid,
    pub build_id: Uuid,
    pub build_version: String,
    pub launcher_version: String,
    /// Optional mods that were on at launch.
    #[serde(default)]
    pub enabled_optional: Vec<String>,
    pub findings: Vec<IntegrityFinding>,
    /// Separates "nothing found" from "the check never ran".
    pub checked_files: u32,
    /// Something turned up that the game must not be launched with.
    #[serde(default)]
    pub block_launch: bool,
}

/// Sent without asking the player, because none of it is personal — versions,
/// hardware, and how fast the master answers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    pub launcher_version: String,
    pub os: String,
    pub arch: String,
    /// The Java the game runs on, once it's installed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java_path: Option<String>,
    /// Megabytes free on the volume holding the launcher directory.
    #[serde(default)]
    pub disk_free_mb: u64,
    /// Megabytes the launcher directory takes up.
    #[serde(default)]
    pub data_size_mb: u64,
    /// `None` means the master didn't answer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub master_ping_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sync_error: Option<String>,
    /// Installed builds, as (server, version).
    #[serde(default)]
    pub instances: Vec<(String, String)>,
}

/// What the master can ask the launcher to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteAction {
    VerifyIntegrity,
    /// Assets are re-downloadable and usually the thing that's corrupt.
    ClearAssetCache,
    ReinstallBuild,
    RestartLauncher,
    KillGame,
}

impl RemoteAction {
    /// Anything that deletes files or interrupts the player needs a prompt.
    /// Verification changes nothing, so it runs silently.
    pub fn needs_confirmation(self) -> bool {
        !matches!(self, RemoteAction::VerifyIntegrity)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            RemoteAction::VerifyIntegrity => "verify_integrity",
            RemoteAction::ClearAssetCache => "clear_asset_cache",
            RemoteAction::ReinstallBuild => "reinstall_build",
            RemoteAction::RestartLauncher => "restart_launcher",
            RemoteAction::KillGame => "kill_game",
        }
    }
}
