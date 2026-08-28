//! `BuildManifest`: what the master tells the launcher to sync.

use crate::manifest_args::ManifestArg;
use crate::server::Modloader;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FileSide {
    #[default]
    Both,
    Client,
    Server,
}

impl FileSide {
    pub fn needed_on_client(&self) -> bool {
        matches!(self, FileSide::Both | FileSide::Client)
    }
}

/// A file the launcher must have at exactly this SHA1.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEntry {
    /// Relative to the instance root: `mods/jei.jar`.
    pub path: String,
    pub sha1: String,
    pub size: u64,
    /// Usually `/files/{sha1}` on the master.
    pub url: String,
    #[serde(default)]
    pub side: FileSide,
    /// Java binaries on unix need `chmod +x`.
    #[serde(default)]
    pub executable: bool,
    /// Which platform this file is for, e.g. `windows-x86_64`. `None` means all.
    ///
    /// The Java runtime and natives are a different binary per OS. Untagged, a
    /// build shipped whichever runtime the master itself runs on, and the JVM
    /// wouldn't start anywhere else.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
}

impl FileEntry {
    pub fn matches_platform(&self) -> bool {
        self.platform
            .as_deref()
            .is_none_or(|p| p == crate::current_platform())
    }
}

/// Drives the sync progress stages and classpath assembly — libraries and mods
/// go on the classpath, assets don't.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    ClientJar,
    Library,
    Runtime,
    Native,
    Asset,
    AssetIndex,
    Java,
    Mod,
    Config,
    Other,
}

/// Condition under which an optional mod switches itself on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModTrigger {
    Always,
    RequiresPermission(String),
}

/// A group of files the player can toggle on and off.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OptionalMod {
    pub name: String,
    pub description: String,
    /// Free-form grouping in the UI: "Performance", "Interface", "Gameplay".
    pub category: String,
    /// Paths of this mod's files; all of them are also in `verified_files`.
    pub files: Vec<String>,
    pub enabled_by_default: bool,
    pub visible: bool,
    /// Requires the `noro.optional.<server_id>.<name>` permission.
    pub limited: bool,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub triggers: Vec<ModTrigger>,
    /// `windows`, `macos`, `linux`. Empty means all of them.
    ///
    /// Matters for mods carrying native libraries: on the wrong OS they don't
    /// just do nothing, they kill the launch, and all the player sees is the
    /// game closing.
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

impl OptionalMod {
    /// `os` is a `std::env::consts::OS` value — the launcher's platform strings
    /// without the architecture half.
    pub fn runs_on(&self, os: &str) -> bool {
        self.os.is_empty()
            || self
                .os
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(os))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecommendedClientSettings {
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub jvm_flags: String,
    pub show_console_on_launch: bool,
    #[serde(default)]
    pub fullscreen: bool,
}

impl Default for RecommendedClientSettings {
    fn default() -> Self {
        Self {
            memory_min_mb: 2048,
            memory_max_mb: 4096,
            jvm_flags: String::new(),
            show_console_on_launch: true,
            fullscreen: false,
        }
    }
}

fn default_allow_optional_mod_suggestions() -> bool {
    true
}

/// Signed with the master's ed25519 key; the public half is compiled into the
/// launcher binary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildManifest {
    pub build_id: Uuid,
    pub server_id: Uuid,
    pub version: String,
    pub mc_version: String,
    pub modloader: Modloader,
    pub modloader_version: Option<String>,
    /// From the modloader's `version.json`.
    pub main_class: String,
    /// Extra JVM arguments from `version.json` — Forge's module flags and such.
    #[serde(default)]
    pub jvm_args: Vec<ManifestArg>,
    /// Game arguments with placeholders, e.g. `--tweakClass`.
    #[serde(default)]
    pub game_args: Vec<ManifestArg>,
    /// Asset index name: `1.21` or `legacy`.
    pub assets_index_name: String,

    /// Always checked against their SHA1 and overwritten. Anything else found in
    /// these directories is deleted.
    pub verified_files: Vec<FileEntry>,

    /// Keyed by `verified_files` path. Feeds progress stages and the classpath.
    #[serde(default)]
    pub artifact_kinds: std::collections::BTreeMap<String, ArtifactKind>,

    /// Never touched: saves, screenshots, options.txt.
    pub unmanaged_paths: Vec<String>,
    /// Ordered rules (§10.6); the last match wins. Empty means the manifest came
    /// from a master too old to send them, and rules get derived from the two
    /// lists above instead.
    #[serde(default)]
    pub path_rules: Vec<crate::path_rules::PathRule>,
    /// Inside the signature, or the list could be swapped out on the client.
    /// Outranks every path rule including `unmanaged`.
    #[serde(default)]
    pub blocked_files: Vec<crate::blocklist::BlockedFile>,

    /// The player may add files here; they don't get deleted.
    pub user_managed_paths: Vec<String>,

    pub optional_mods: Vec<OptionalMod>,

    #[serde(default = "default_allow_optional_mod_suggestions")]
    pub allow_optional_mod_suggestions: bool,

    #[serde(default)]
    pub recommended_client_settings: RecommendedClientSettings,

    /// ed25519 over the manifest serialized with this field empty.
    #[serde(default, with = "serde_bytes_vec")]
    pub signature: Vec<u8>,
}

impl BuildManifest {
    /// Both signing and verification go through here. serde_json is
    /// deterministic for a given struct, so the two sides see the same bytes.
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut clone = self.clone();
        clone.signature = Vec::new();
        serde_json::to_vec(&clone).expect("BuildManifest is always serializable")
    }

    /// Denominator for the progress bar.
    pub fn total_client_size(&self) -> u64 {
        self.verified_files
            .iter()
            .filter(|f| f.side.needed_on_client())
            .map(|f| f.size)
            .sum()
    }

    pub fn kind_of(&self, path: &str) -> ArtifactKind {
        self.artifact_kinds
            .get(path)
            .copied()
            .unwrap_or(ArtifactKind::Other)
    }
}

/// `Vec<u8>` as a JSON array of numbers, so this doesn't depend on serde's
/// base64 feature being on.
mod serde_bytes_vec {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(bytes.iter().copied())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        Vec::deserialize(d)
    }
}

#[cfg(test)]
mod optional_mod_tests {
    use super::*;

    fn mod_for(os: &[&str]) -> OptionalMod {
        OptionalMod {
            name: "sodium".into(),
            description: String::new(),
            category: String::new(),
            files: vec![],
            enabled_by_default: false,
            visible: true,
            limited: false,
            dependencies: vec![],
            conflicts: vec![],
            triggers: vec![],
            os: os.iter().map(|s| s.to_string()).collect(),
            icon_url: None,
            author: None,
        }
    }

    /// Every build made before the field existed has an empty list, and those
    /// mods must not silently disappear from the player's list.
    #[test]
    fn empty_list_means_every_system() {
        assert!(mod_for(&[]).runs_on("windows"));
        assert!(mod_for(&[]).runs_on("linux"));
    }

    #[test]
    fn keeps_only_listed_systems() {
        let windows_only = mod_for(&["windows"]);
        assert!(windows_only.runs_on("windows"));
        assert!(!windows_only.runs_on("macos"));
    }

    /// The list is typed by hand in the admin panel.
    #[test]
    fn ignores_letter_case() {
        assert!(mod_for(&["Windows"]).runs_on("windows"));
    }
}
