//! Launcher configuration, persisted to disk.

use schema::RecommendedClientSettings;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

/// Every field falls back to its default when missing: a version that adds or
/// renames a field must not turn an existing config into a parse error, which
/// would reset everything the player set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LauncherConfig {
    pub master_url: String,
    #[serde(default = "default_locale")]
    pub locale: String,
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    /// Space-separated, passed to the JVM as-is.
    pub jvm_flags: String,
    pub show_console_on_launch: bool,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default = "default_crash_reports")]
    pub crash_reports: bool,
    /// Rich Presence: the server and what the player is doing, shown to their
    /// Discord friends. On by default, as it has always been.
    pub discord_rpc: bool,
    /// Per-server overrides for the fields above.
    #[serde(default)]
    pub server_settings: BTreeMap<Uuid, ServerClientSettings>,
    /// Build pinned per server. No entry means whatever the master currently
    /// publishes, which is the default.
    #[serde(default)]
    pub selected_build: BTreeMap<Uuid, Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerClientSettings {
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    /// The player's own flags. The build's are not kept here: they come with
    /// every manifest and are added at launch, so a saved override can't drop
    /// the ones the build does not start without.
    pub jvm_flags: String,
    pub show_console_on_launch: bool,
    #[serde(default)]
    pub fullscreen: bool,
}

const DEFAULT_MEMORY_MIN_MB: u32 = 2048;
const DEFAULT_MEMORY_MAX_MB: u32 = 4096;

impl Default for ServerClientSettings {
    fn default() -> Self {
        Self {
            memory_min_mb: DEFAULT_MEMORY_MIN_MB,
            memory_max_mb: DEFAULT_MEMORY_MAX_MB,
            jvm_flags: String::new(),
            show_console_on_launch: true,
            fullscreen: false,
        }
    }
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            master_url: default_master_url(),
            locale: default_locale(),
            memory_min_mb: DEFAULT_MEMORY_MIN_MB,
            memory_max_mb: DEFAULT_MEMORY_MAX_MB,
            jvm_flags: String::new(),
            show_console_on_launch: true,
            fullscreen: false,
            crash_reports: default_crash_reports(),
            discord_rpc: true,
            server_settings: BTreeMap::new(),
            selected_build: BTreeMap::new(),
        }
    }
}

/// Opt-out rather than opt-in — without it we hear about crashes only when
/// someone writes in.
fn default_crash_reports() -> bool {
    true
}

fn default_locale() -> String {
    let system = sys_locale::get_locale()
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_default();
    locale_for(&system).to_string()
}

/// ru and en are the only two we ship, so everything else lands on en.
/// Accepts both `ru-RU` (Windows, macOS) and `ru_RU.UTF-8` (Unix).
fn locale_for(system: &str) -> &'static str {
    if system.to_ascii_lowercase().starts_with("ru") {
        "ru"
    } else {
        "en"
    }
}

/// The master address is baked in at build time and is mandatory for release
/// builds — `noro_launcher::verify` enforces that. The literal fallback below
/// stays a dev address on purpose, so a build without one can't quietly talk to
/// production.
fn default_master_url() -> String {
    stamped_master_url().unwrap_or_else(|| {
        option_env!("NORO_MASTER_URL")
            .unwrap_or("http://localhost:8080")
            .to_string()
    })
}

/// The address the bootstrapper handed over: in the environment when it starts
/// core, or in `bootstrap.json`, which it rewrites from its own stamp on every
/// launch.
///
/// `None` means core was started on its own — that happens in development, and
/// there the saved address is the only one there is.
fn stamped_master_url() -> Option<String> {
    if let Ok(val) = std::env::var("NORO_MASTER_URL") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if cfg!(debug_assertions) {
        return None;
    }
    let boot_path = crate::directories::LauncherDirectories::new().bootstrap_file();
    let raw = std::fs::read_to_string(&boot_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    v.get("master_url")?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

impl LauncherConfig {
    pub fn default_client_settings(&self) -> ServerClientSettings {
        ServerClientSettings {
            memory_min_mb: self.memory_min_mb,
            memory_max_mb: self.memory_max_mb,
            jvm_flags: self.jvm_flags.clone(),
            show_console_on_launch: self.show_console_on_launch,
            fullscreen: self.fullscreen,
        }
    }

    pub fn settings_for_server(
        &self,
        server_id: &Uuid,
        recommended: Option<&RecommendedClientSettings>,
    ) -> ServerClientSettings {
        if let Some(saved) = self.server_settings.get(server_id) {
            return saved.clone();
        }
        let mut settings = self.default_client_settings();
        if let Some(recommended) = recommended {
            settings.memory_min_mb = recommended.memory_min_mb;
            settings.memory_max_mb = recommended.memory_max_mb;
            settings.show_console_on_launch = recommended.show_console_on_launch;
            settings.fullscreen = recommended.fullscreen;
        }
        settings
    }

    pub fn launch_config_for_server(
        &self,
        server_id: &Uuid,
        recommended: &RecommendedClientSettings,
    ) -> Self {
        let settings = self.settings_for_server(server_id, Some(recommended));
        let mut config = self.clone();
        config.memory_min_mb = settings.memory_min_mb;
        // A recommendation with min above max would stop the JVM from starting.
        config.memory_max_mb = settings.memory_max_mb.max(settings.memory_min_mb);
        config.jvm_flags = launch_jvm_flags(&settings.jvm_flags, &recommended.jvm_flags);
        config.show_console_on_launch = settings.show_console_on_launch;
        config.fullscreen = settings.fullscreen;
        config
    }

    pub fn set_server_memory(&mut self, server_id: Uuid, min_mb: u32, max_mb: u32) {
        let defaults = self.default_client_settings();
        let settings = self.server_settings.entry(server_id).or_insert(defaults);
        settings.memory_min_mb = min_mb;
        settings.memory_max_mb = max_mb.max(min_mb);
    }

    pub fn set_server_jvm_flags(&mut self, server_id: Uuid, flags: String) {
        let defaults = self.default_client_settings();
        self.server_settings
            .entry(server_id)
            .or_insert(defaults)
            .jvm_flags = flags;
    }

    pub fn set_server_console(&mut self, server_id: Uuid, enabled: bool) {
        let defaults = self.default_client_settings();
        self.server_settings
            .entry(server_id)
            .or_insert(defaults)
            .show_console_on_launch = enabled;
    }

    pub fn set_server_fullscreen(&mut self, server_id: Uuid, enabled: bool) {
        let defaults = self.default_client_settings();
        self.server_settings
            .entry(server_id)
            .or_insert(defaults)
            .fullscreen = enabled;
    }

    pub fn reset_server_settings(&mut self, server_id: &Uuid) {
        self.server_settings.remove(server_id);
    }

    pub fn ws_url(&self) -> String {
        let base = self.master_url.trim_end_matches('/');
        let ws = base
            .replacen("https://", "wss://", 1)
            .replacen("http://", "ws://", 1)
            // macOS resolves localhost to ::1 first and the connection hangs
            // there while a v4-only master sits waiting.
            .replace("localhost", "127.0.0.1");
        format!("{ws}/ws/launcher")
    }

    /// Take the address the bootstrapper stamped, replacing the saved one.
    /// Returns the address left behind, if it changed.
    ///
    /// Where the master lives is the build's business, not the player's: there
    /// is no setting for it, and `config.json` only keeps it because the whole
    /// config is written out as one file. A launcher installed from a new
    /// address has to go to that address — before this, it kept knocking on the
    /// one written down the first time it ever started, and the only cure was
    /// deleting the config by hand.
    pub fn adopt_stamped_master(&mut self) -> Option<String> {
        if cfg!(debug_assertions) && std::env::var_os("NORO_MASTER_URL").is_none() {
            let dev_url = "http://127.0.0.1:8080";
            if self.master_url != dev_url {
                return Some(std::mem::replace(&mut self.master_url, dev_url.to_string()));
            }
            return None;
        }
        let stamped = stamped_master_url()?;
        if stamped == self.master_url {
            return None;
        }
        Some(std::mem::replace(&mut self.master_url, stamped))
    }

    /// Same substitution as [`Self::ws_url`], applied to configs written before
    /// it existed. Returns whether anything changed.
    pub fn fix_localhost(&mut self) -> bool {
        if self.master_url.contains("://localhost") {
            self.master_url = self.master_url.replace("://localhost", "://127.0.0.1");
            true
        } else {
            false
        }
    }
}

/// The player's flags first and the build's last: on a clash — a `-D` set
/// twice, two `-Xmx` — the JVM keeps the later one, and the build's are the
/// ones it can't start without (GTNH's system class loader, for one).
fn launch_jvm_flags(player: &str, build: &str) -> String {
    player
        .split_whitespace()
        .chain(build.split_whitespace())
        .collect::<Vec<_>>()
        .join(" ")
}

impl From<&ServerClientSettings> for bridge::ClientSettingsState {
    fn from(s: &ServerClientSettings) -> Self {
        Self {
            memory_min_mb: s.memory_min_mb,
            memory_max_mb: s.memory_max_mb,
            jvm_flags: s.jvm_flags.clone(),
            show_console_on_launch: s.show_console_on_launch,
            fullscreen: s.fullscreen,
        }
    }
}

/// Kept in its own file, not in [`LauncherConfig`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptionalModsSelection {
    /// server id → enabled mod names.
    pub enabled: BTreeMap<Uuid, Vec<String>>,
}

impl OptionalModsSelection {
    /// `None` until the player has toggled something for this server. An empty
    /// list is a real choice — every optional mod off — and must not fall back
    /// to the build's defaults.
    pub fn for_server(&self, server_id: &Uuid) -> Option<Vec<String>> {
        self.enabled.get(server_id).cloned()
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
