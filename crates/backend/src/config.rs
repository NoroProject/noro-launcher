//! Конфигурация лаунчера (сохраняется на диск).

use schema::RecommendedClientSettings;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    /// URL мастер-сервера.
    pub master_url: String,
    /// Код языка интерфейса.
    #[serde(default = "default_locale")]
    pub locale: String,
    /// Минимум памяти JVM (МБ).
    pub memory_min_mb: u32,
    /// Максимум памяти JVM (МБ).
    pub memory_max_mb: u32,
    /// Доп. JVM-флаги (через пробел).
    pub jvm_flags: String,
    /// Открывать ли окно консоли при запуске игры.
    pub show_console_on_launch: bool,
    /// Запускать ли игры в полноэкранном режиме.
    #[serde(default)]
    pub fullscreen: bool,
    /// Отправлять ли отчёты о падениях. Игрок может отказаться — см. telemetry.
    #[serde(default = "default_crash_reports")]
    pub crash_reports: bool,
    /// Персональные настройки клиента для конкретных серверов.
    #[serde(default)]
    pub server_settings: BTreeMap<Uuid, ServerClientSettings>,
    /// Выбранная версия сборки по серверам. Нет записи — берётся текущая
    /// опубликованная, то есть поведение по умолчанию не меняется.
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

/// По умолчанию включено: иначе о падениях мы не узнаём вовсе.
fn default_crash_reports() -> bool {
    true
}

fn default_locale() -> String {
    // Системный язык, если мы его поддерживаем, иначе английский.
    std::env::var("LANG")
        .ok()
        .and_then(|l| l.split('.').next().map(str::to_string))
        .filter(|l| l.starts_with("ru"))
        .map(|_| "ru".to_string())
        .unwrap_or_else(|| "en".to_string())
}

/// Адрес мастера вшивается на сборке. В release он обязателен — за этим следит
/// `noro_launcher::verify`. Здесь остаётся только dev-адрес: подставлять сюда
/// боевой домен значило бы, что отладочная сборка молча ходит в прод.
fn default_master_url() -> String {
    if let Ok(val) = std::env::var("NORO_MASTER_URL") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    let boot_path = crate::directories::LauncherDirectories::new().bootstrap_file();
    if let Ok(raw) = std::fs::read_to_string(&boot_path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(url) = v.get("master_url").and_then(|u| u.as_str()) {
                let trimmed = url.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    option_env!("NORO_MASTER_URL")
        .unwrap_or("http://localhost:8080")
        .to_string()
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

    /// WebSocket URL мастера.
    pub fn ws_url(&self) -> String {
        let base = self.master_url.trim_end_matches('/');
        let ws = base
            .replacen("https://", "wss://", 1)
            .replacen("http://", "ws://", 1)
            .replace("localhost", "127.0.0.1");
        format!("{ws}/ws/launcher")
    }

    /// Миграция: заменить localhost на 127.0.0.1 для избежания проблем с IPv6 на macOS.
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

/// Выбор опциональных модов по серверам (отдельный persistent-файл).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptionalModsSelection {
    /// server_id → список включённых имён модов.
    pub enabled: BTreeMap<Uuid, Vec<String>>,
}

impl OptionalModsSelection {
    pub fn for_server(&self, server_id: &Uuid) -> Vec<String> {
        self.enabled.get(server_id).cloned().unwrap_or_default()
    }
}
