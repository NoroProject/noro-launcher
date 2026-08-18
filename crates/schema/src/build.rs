//! BuildManifest — главный документ синхронизации между мастером и лаунчером.

use crate::manifest_args::ManifestArg;
use crate::server::Modloader;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// На какой стороне нужен файл.
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

/// Один файл, который лаунчер обязан иметь с точным SHA1.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileEntry {
    /// Относительный путь от корня игровой папки: "mods/jei.jar".
    pub path: String,
    pub sha1: String,
    pub size: u64,
    /// URL для скачивания с мастера (обычно /files/{sha1}).
    pub url: String,
    #[serde(default)]
    pub side: FileSide,
    /// Исполняемый ли файл (для java-бинарников на unix нужен chmod +x).
    #[serde(default)]
    pub executable: bool,
    /// Платформа, которой файл предназначен ("windows-x86_64"). `None` — всем.
    ///
    /// Java-рантайм и natives — разные бинарники под каждую ОС. Без пометки
    /// сборка несла рантайм только той платформы, на которой крутится мастер, и
    /// на остальных JVM не запускалась.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
}

impl FileEntry {
    /// Нужен ли файл этой машине. Java и natives помечены платформой, остальное
    /// одинаково всюду.
    pub fn matches_platform(&self) -> bool {
        self.platform
            .as_deref()
            .is_none_or(|p| p == crate::current_platform())
    }
}

/// Категория артефакта — помогает лаунчеру понимать стадию синхронизации и
/// собирать classpath (libraries попадают в classpath, mods тоже, assets — нет).
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

/// Триггер опционального мода — авто-включение по условию (например, наличие GPU).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModTrigger {
    /// Включить по умолчанию для всех.
    Always,
    /// Включить только если есть право.
    RequiresPermission(String),
}

/// Опциональный мод — набор файлов, которые пользователь может включить/выключить.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OptionalMod {
    pub name: String,
    pub description: String,
    /// "Производительность", "Интерфейс", "Геймплей".
    pub category: String,
    /// Пути файлов этого мода (они присутствуют в verified_files).
    pub files: Vec<String>,
    pub enabled_by_default: bool,
    pub visible: bool,
    /// Требует право `noro.optional.<server_id>.<name>`.
    pub limited: bool,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub triggers: Vec<ModTrigger>,
    /// Системы, где мод имеет смысл: `windows`, `macos`, `linux`. Пусто — все.
    ///
    /// Нужно модам с нативными библиотеками: на чужой системе такой мод не
    /// просто бесполезен, он роняет запуск, а игрок видит только «игра
    /// закрылась» и чинить это ему нечем.
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

impl OptionalMod {
    /// Годится ли мод для этой системы. Пустой список — годится везде.
    ///
    /// Сравнение по `std::env::consts::OS`: `windows`, `macos`, `linux` — та же
    /// нотация, что и в платформах лаунчера, только без архитектуры.
    pub fn runs_on(&self, os: &str) -> bool {
        self.os.is_empty()
            || self
                .os
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(os))
    }
}

/// Рекомендованные клиентские настройки для конкретной сборки.
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

/// Главный документ синхронизации. Подписывается ed25519 ключом мастера;
/// публичный ключ зашит в бинарник лаунчера.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildManifest {
    pub build_id: Uuid,
    pub server_id: Uuid,
    pub version: String,
    pub mc_version: String,
    pub modloader: Modloader,
    pub modloader_version: Option<String>,
    /// Главный класс для запуска JVM (берётся из version.json модлоадера).
    pub main_class: String,
    /// Дополнительные JVM-аргументы из version.json (модульные флаги Forge и т.п.).
    #[serde(default)]
    pub jvm_args: Vec<ManifestArg>,
    /// Game-аргументы (--tweakClass и т.п.) с плейсхолдерами.
    #[serde(default)]
    pub game_args: Vec<ManifestArg>,
    /// Имя версии ассетов ("1.21" или "legacy").
    pub assets_index_name: String,

    /// Файлы, которые ВСЕГДА сверяются по SHA1 и перезаписываются.
    /// Лишние файлы в этих директориях УДАЛЯЮТСЯ.
    pub verified_files: Vec<FileEntry>,

    /// Привязка путей к категориям (для стадий прогресса и classpath).
    /// Ключ — path из verified_files.
    #[serde(default)]
    pub artifact_kinds: std::collections::BTreeMap<String, ArtifactKind>,

    /// Пути, которые лаунчер НЕ ТРОГАЕТ (saves, screenshots, options.txt).
    pub unmanaged_paths: Vec<String>,
    /// Единый упорядоченный список правил (§10.6). Побеждает последнее
    /// совпавшее. Пусто — манифест от мастера, который ещё не умеет их слать:
    /// тогда правила выводятся из двух списков выше.
    #[serde(default)]
    pub path_rules: Vec<crate::path_rules::PathRule>,
    /// Запрещённые файлы. Едут внутри подписи — иначе список подменяется на
    /// клиенте. Приоритет выше всех правил путей, включая `unmanaged`: папка
    /// ресурспаков не синхронизируется, но xray оттуда удаляется.
    #[serde(default)]
    pub blocked_files: Vec<crate::blocklist::BlockedFile>,

    /// Пути, где пользователь МОЖЕТ добавлять файлы (не удаляются).
    pub user_managed_paths: Vec<String>,

    pub optional_mods: Vec<OptionalMod>,

    #[serde(default = "default_allow_optional_mod_suggestions")]
    pub allow_optional_mod_suggestions: bool,

    #[serde(default)]
    pub recommended_client_settings: RecommendedClientSettings,

    /// ed25519-подпись над канонической сериализацией манифеста БЕЗ этого поля.
    #[serde(default, with = "serde_bytes_vec")]
    pub signature: Vec<u8>,
}

impl BuildManifest {
    /// Байты для подписи/проверки: тот же манифест, но с пустой подписью,
    /// сериализованный в canonical JSON (serde_json детерминирован по структуре).
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut clone = self.clone();
        clone.signature = Vec::new();
        serde_json::to_vec(&clone).expect("BuildManifest всегда сериализуется")
    }

    /// Суммарный размер всех клиентских файлов (для прогресс-бара).
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

/// serde-хелпер: Vec<u8> как массив чисел (надёжно для JSON, не зависит от base64-фичи).
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

    /// Без списка систем мод годится везде: так лежат все сборки, сделанные до
    /// появления поля, и молча спрятать их от игроков нельзя.
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

    /// Регистр приходит из админки, где его набирают руками.
    #[test]
    fn ignores_letter_case() {
        assert!(mod_for(&["Windows"]).runs_on("windows"));
    }
}
