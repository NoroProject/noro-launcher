//! Сообщения, пересекающие границу frontend ↔ backend.

use crate::modal_action::ModalAction;
use schema::{LauncherVersion, NewsItem, NotifLevel, ServerEntry, UserProfile};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ClientSettingsState {
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub jvm_flags: String,
    pub show_console_on_launch: bool,
}

/// Frontend → Backend.
#[derive(Debug)]
pub enum MessageToBackend {
    // --- Авторизация (Discord OAuth через браузер) ---
    StartDiscordLogin {
        modal_action: ModalAction,
    },
    Logout,

    // --- Серверы / контент ---
    RequestServerList,
    RequestNews,
    /// Открыть карточку сервера — backend подтянет манифест и пришлёт опц. моды.
    OpenServer {
        server_id: Uuid,
    },
    LaunchServer {
        server_id: Uuid,
        modal_action: ModalAction,
    },
    KillGame {
        server_id: Uuid,
    },
    SetOptionalMods {
        server_id: Uuid,
        enabled: Vec<String>,
    },
    SuggestOptionalMod {
        server_id: Uuid,
        build_id: Option<Uuid>,
        provider: String,
        project_id: String,
        title: String,
        icon_url: Option<String>,
        description: Option<String>,
    },

    // --- Настройки ---
    SetMemory {
        min_mb: u32,
        max_mb: u32,
    },
    SetJvmFlags {
        flags: String,
    },
    SetShowConsoleOnLaunch {
        enabled: bool,
    },
    SetServerMemory {
        server_id: Uuid,
        min_mb: u32,
        max_mb: u32,
    },
    SetServerJvmFlags {
        server_id: Uuid,
        flags: String,
    },
    SetServerShowConsoleOnLaunch {
        server_id: Uuid,
        enabled: bool,
    },
    ResetServerClientSettings {
        server_id: Uuid,
    },
    OpenServerClientFolder {
        server_id: Uuid,
    },
    /// Сменить язык интерфейса: backend сохранит выбор и подтянет каталог.
    SetLocale {
        code: String,
    },

    // --- Обновление лаунчера ---
    InstallUpdate {
        version: LauncherVersion,
        modal_action: ModalAction,
    },

    /// Нативная загрузка скина из лаунчера (без браузера).
    UploadSkin {
        bytes: Vec<u8>,
    },

    /// Второй процесс попросил показать окно (single-instance).
    FocusWindow,

    Quit,
}

/// Стадии синхронизации — пользователь видит детальный прогресс.
///
/// Порядок вариантов — это и порядок полос в UI: стадии загрузки идут
/// параллельно, и `Ord` держит их список стабильным, а не в порядке прихода.
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
    /// Человекочитаемое описание для UI.
    pub fn label(&self) -> &'static str {
        match self {
            SyncStage::CheckingFiles => "Checking files...",
            SyncStage::DownloadingJava => "Downloading Java...",
            SyncStage::DownloadingMinecraft => "Downloading Minecraft...",
            SyncStage::DownloadingLibraries => "Downloading libraries...",
            SyncStage::DownloadingAssets => "Downloading assets...",
            SyncStage::DownloadingMods => "Downloading mods...",
            SyncStage::ApplyingForgePatches => "Applying Forge patches...",
            SyncStage::Cleaning => "Cleaning extra files...",
            SyncStage::Done => "Done",
        }
    }

    /// Короткая метка для строки стадии — рядом с полосой места мало.
    pub fn short_label(&self) -> &'static str {
        match self {
            SyncStage::CheckingFiles => "Checking",
            SyncStage::DownloadingJava => "Java",
            SyncStage::DownloadingMinecraft => "Minecraft",
            SyncStage::DownloadingLibraries => "Libraries",
            SyncStage::DownloadingAssets => "Assets",
            SyncStage::DownloadingMods => "Mods",
            SyncStage::ApplyingForgePatches => "Forge",
            SyncStage::Cleaning => "Cleaning",
            SyncStage::Done => "Done",
        }
    }

    /// Качает ли стадия файлы. У таких прогресс в байтах и своя полоса; у
    /// остальных счётчик в штуках, и сложить их в общий итог нельзя.
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

/// UI-представление опционального мода (с учётом прав пользователя).
#[derive(Debug, Clone)]
pub struct OptionalModInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub icon_url: Option<String>,
    pub author: Option<String>,
    /// Требует права.
    pub limited: bool,
    /// Доступен ли пользователю (есть право, если limited).
    pub allowed: bool,
    /// Включён ли сейчас.
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginErrorKind {
    /// Пользователь закрыл окно браузера / истёк таймаут.
    Cancelled,
    /// Discord/мастер отверг.
    Rejected(String),
    /// Сетевая ошибка.
    Network(String),
}

/// Что лаунчер может сделать со сборкой прямо сейчас.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildState {
    /// Файлов нет — сборку нужно поставить.
    #[default]
    Missing,
    /// Установлена другая версия — нужно обновить.
    Outdated,
    /// Всё на месте, можно играть.
    Ready,
}

/// Backend → Frontend.
#[derive(Debug)]
pub enum MessageToFrontend {
    LoginSuccess {
        user: UserProfile,
    },
    LoginFailed {
        kind: LoginErrorKind,
    },
    LoggedOut,

    ServerList {
        servers: Vec<ServerEntry>,
    },
    NewsUpdated {
        items: Vec<NewsItem>,
    },

    /// Состояние локальной копии сборки — от него зависит подпись главной кнопки.
    BuildStateChanged {
        server_id: Uuid,
        state: BuildState,
    },

    /// Текущая конфигурация лаунчера (для экрана настроек).
    ConfigState {
        memory_min_mb: u32,
        memory_max_mb: u32,
        jvm_flags: String,
        show_console_on_launch: bool,
        master_url: String,
        locale: String,
        server_settings: BTreeMap<Uuid, ClientSettingsState>,
    },
    /// Каталог перевода с мастера (или из локального кеша).
    LocaleCatalog {
        code: String,
        ftl: String,
    },
    /// Опциональные моды сервера (после получения манифеста).
    OptionalMods {
        server_id: Uuid,
        mods: Vec<OptionalModInfo>,
    },
    ServerClientRecommendation {
        server_id: Uuid,
        settings: ClientSettingsState,
    },

    /// Прогресс синхронизации (файлы, java, assets — всё через один канал).
    SyncProgress {
        server_id: Uuid,
        stage: SyncStage,
        done: u64,
        total: u64,
        file: String,
    },
    SyncComplete {
        server_id: Uuid,
    },
    SyncFailed {
        server_id: Uuid,
        reason: String,
    },

    GameStarted {
        server_id: Uuid,
    },
    GameStopped {
        server_id: Uuid,
        exit_ok: bool,
    },
    GameLog {
        server_id: Uuid,
        line: String,
        level: GameLogLevel,
        timestamp: i64,
    },

    LauncherUpdateAvailable {
        version: LauncherVersion,
    },

    /// Уведомление ключом перевода. Текст собирается во фронтенде, где живёт
    /// каталог, — так сообщения от мастера следуют выбранному языку.
    AddNotification {
        key: String,
        args: BTreeMap<String, String>,
        level: NotifLevel,
    },
    /// Аплоад скина не удался — снять индикатор загрузки в профиле.
    SkinUploadFailed,
    PermissionsUpdated {
        user: UserProfile,
    },

    /// Соединение с мастером установлено/потеряно — для индикатора в UI.
    ConnectionState {
        online: bool,
    },

    OpenOrFocusMainWindow,
    CloseModal,
    Quit,
}
