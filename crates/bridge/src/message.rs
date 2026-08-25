//! Сообщения, пересекающие границу frontend ↔ backend.

use crate::modal_action::ModalAction;
use schema::{LauncherVersion, NewsItem, NotifLevel, ServerEntry, UserProfile};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSettingsState {
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub jvm_flags: String,
    pub show_console_on_launch: bool,
    #[serde(default)]
    pub fullscreen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogHitInfo {
    pub provider: String,
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub author: Option<String>,
    pub downloads: u64,
}

/// Страница мода целиком. Приходит отдельным запросом: в выдаче поиска нет ни
/// описания, ни скриншотов, а тянуть их для каждой карточки списка незачем.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModProjectInfo {
    pub provider: String,
    pub project_id: String,
    /// Markdown у Modrinth, HTML у CurseForge.
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub gallery: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    pub source_url: Option<String>,
    pub issues_url: Option<String>,
    pub wiki_url: Option<String>,
    pub page_url: Option<String>,
    pub license: Option<String>,
}

/// Frontend → Backend.
#[derive(Debug)]
pub enum MessageToBackend {
    // --- Авторизация (вход через сайт / ключ доступа) ---
    /// Вход через сайт: способов входа много, и живут они там.
    StartWebLogin {
        modal_action: ModalAction,
    },
    StartKeyLogin {
        key: String,
        modal_action: ModalAction,
    },
    StartBiometricLogin {
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
    /// Выбрать версию сборки для сервера. `None` — вернуться к текущей.
    SelectBuild {
        server_id: uuid::Uuid,
        build_id: Option<uuid::Uuid>,
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
    SearchCatalog {
        query: String,
        provider: String,
        mc_version: Option<String>,
        loader: Option<String>,
        offset: u32,
    },
    RequestModProject {
        provider: String,
        project_id: String,
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
    SetFullscreen {
        enabled: bool,
    },
    /// Игрок разрешил или запретил отправку отчётов о падениях.
    SetCrashReports {
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
    SetServerFullscreen {
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

    /// Ответ на предложенное действие.
    RemoteActionAnswer {
        action: schema::RemoteAction,
        server_id: Option<Uuid>,
        accepted: bool,
    },

    /// Ответ на запрос логов.
    LogRequestAnswer {
        request_id: Uuid,
        accepted: bool,
    },

    /// Ответ на диалог impersonation.
    ImpersonateAnswer {
        grant_id: Uuid,
        accepted: bool,
    },
    /// Выйти из чужого аккаунта обратно в свой.
    ImpersonateExit,

    /// «Сообщить о проблеме»: собрать логи и отправить мастеру.
    ///
    /// Инициатива игрока, а не запрос админа — ни согласия, ни гранта здесь не
    /// нужно, он сам нажал кнопку.
    SendSupportBundle {
        server_id: Option<Uuid>,
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

    /// Сменить модель уже загруженного скина: тонкая (Алекс) или классическая
    /// (Стив). Отдельно от загрузки — картинка при этом не меняется, а
    /// требовать от игрока исходник ради ширины рук не за что.
    SetSkinModel {
        slim: bool,
    },

    RequestCapesList,
    RequestSkinPresetsList,
    SelectCape {
        cape_id: Option<Uuid>,
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
    /// Моды, с которыми этот несовместим: включить оба нельзя.
    pub conflicts: Vec<String>,
    /// Моды, без которых этот не имеет смысла.
    pub dependencies: Vec<String>,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerSkinPresetItem {
    pub id: String,
    pub name: String,
    pub skin_url: String,
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
        fullscreen: bool,
        crash_reports: bool,
        /// Вшит ли DSN в эту сборку. Без него переключатель показывать незачем.
        crash_reports_available: bool,
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
        allow_suggestions: bool,
        installed_files: Vec<String>,
    },
    ServerClientRecommendation {
        server_id: Uuid,
        settings: ClientSettingsState,
    },
    CatalogSearchResults {
        hits: Vec<CatalogHitInfo>,
        total: u32,
        offset: u32,
        limit: u32,
    },
    /// Каталог не ответил. Раньше в этом случае не приходило ничего, и экран
    /// навсегда оставался в состоянии «ищем моды».
    CatalogFailed {
        message: String,
    },
    /// Полная страница мода — ответ на `RequestModProject`.
    ModProjectLoaded {
        project: ModProjectInfo,
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
    /// Паки и шейдеры обновились, пока игра запущена.
    ///
    /// Отдельно от `SyncComplete`: та говорит «сборка готова к запуску», а эта —
    /// «в запущенной игре появилось новое, применить можно перезагрузкой
    /// ресурсов». Файлы, которые игра держала открытыми, встанут при следующем
    /// запуске, и о них тоже надо сказать — иначе человек будет ждать того,
    /// чего не произошло.
    LiveSynced {
        server_id: Uuid,
        updated: Vec<String>,
        locked: Vec<String>,
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
    CapesList {
        capes: Vec<schema::CapeRow>,
    },
    SkinPresetsList {
        presets: Vec<ServerSkinPresetItem>,
    },

    /// Соединение с мастером установлено/потеряно — для индикатора в UI.
    ConnectionState {
        online: bool,
    },

    /// Админ нажал «Login as» в вебе — спросить подтверждение здесь.
    ///
    /// Нативный диалог это второй фактор: он закрывает случай «злоумышленник
    /// получил веб-сессию админа, но не доступ к его машине».
    ImpersonatePrompt {
        grant_id: Uuid,
        actor_username: String,
        target_username: String,
        reason: String,
        /// Сколько секунд осталось на решение.
        expires_in_secs: i64,
    },
    /// Админ просит выполнить действие. Игрок решает.
    RemoteActionPrompt {
        action: schema::RemoteAction,
        server_id: Option<Uuid>,
        actor_username: String,
    },

    /// Админ просит логи. Игрок решает, отправлять ли.
    LogRequestPrompt {
        request_id: Uuid,
        actor_username: String,
        reason: String,
        /// Собран без спроса: логи уже уехали, модалка только сообщает.
        forced: bool,
        /// Что именно уйдёт — уже очищенный текст.
        preview: String,
        /// Имена файлов и их размер на диске.
        files: Vec<(String, u64)>,
    },

    /// Сессия impersonation началась или закончилась — для баннера в лаунчере.
    ImpersonationChanged {
        /// `None` — вернулись в свой аккаунт.
        as_username: Option<String>,
    },

    OpenOrFocusMainWindow,
    CloseModal,
    Quit,
}
