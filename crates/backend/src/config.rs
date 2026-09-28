//! Launcher configuration, persisted to disk.

use schema::RecommendedClientSettings;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Per-server overrides for the fields above.
    #[serde(default)]
    pub server_settings: BTreeMap<Uuid, ServerClientSettings>,
    /// Build pinned per server. No entry means whatever the master currently
    /// publishes, which is the default.
    #[serde(default)]
    pub selected_build: BTreeMap<Uuid, Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerClientSettings {
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub jvm_flags: String,
    pub show_console_on_launch: bool,
    #[serde(default)]
    pub fullscreen: bool,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            master_url: default_master_url(),
            locale: default_locale(),
            memory_min_mb: 2048,
            memory_max_mb: 4096,
            jvm_flags: String::new(),
            show_console_on_launch: true,
            fullscreen: false,
            crash_reports: default_crash_reports(),
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
    // ru and en are the only two we ship, so everything else lands on en.
    std::env::var("LANG")
        .ok()
        .and_then(|l| l.split('.').next().map(str::to_string))
        .filter(|l| l.starts_with("ru"))
        .map(|_| "ru".to_string())
        .unwrap_or_else(|| "en".to_string())
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
        self.server_settings
            .get(server_id)
            .cloned()
            .unwrap_or_else(|| {
                recommended
                    .map(ServerClientSettings::from)
                    .unwrap_or_else(|| self.default_client_settings())
            })
    }

    pub fn launch_config_for_server(
        &self,
        server_id: &Uuid,
        recommended: &RecommendedClientSettings,
    ) -> Self {
        let settings = self.settings_for_server(server_id, Some(recommended));
        let mut config = self.clone();
        config.memory_min_mb = settings.memory_min_mb;
        config.memory_max_mb = settings.memory_max_mb;
        config.jvm_flags = settings.jvm_flags;
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

impl From<&RecommendedClientSettings> for ServerClientSettings {
    fn from(value: &RecommendedClientSettings) -> Self {
        Self {
            memory_min_mb: value.memory_min_mb,
            memory_max_mb: value.memory_max_mb,
            jvm_flags: value.jvm_flags.clone(),
            show_console_on_launch: value.show_console_on_launch,
            fullscreen: value.fullscreen,
        }
    }
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
    pub fn for_server(&self, server_id: &Uuid) -> Vec<String> {
        self.enabled.get(server_id).cloned().unwrap_or_default()
    }
}
