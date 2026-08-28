//! Everything that crosses the frontend ↔ backend boundary.

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

/// The full mod page, fetched on demand: search results carry neither the
/// description nor the screenshots, and pulling them for every card in a list
/// would be pointless.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModProjectInfo {
    pub provider: String,
    pub project_id: String,
    /// Markdown from Modrinth, HTML from CurseForge.
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
    // --- Auth ---
    /// The site owns the login methods, so this hands off to a browser.
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

    // --- Servers and content ---
    RequestServerList,
    RequestNews,
    /// The backend pulls the manifest and answers with `OptionalMods`.
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
    /// `None` goes back to whatever the server currently ships.
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

    // --- Settings ---
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
    /// The backend stores the choice and fetches the catalog for it.
    SetLocale {
        code: String,
    },

    RemoteActionAnswer {
        action: schema::RemoteAction,
        server_id: Option<Uuid>,
        accepted: bool,
    },

    LogRequestAnswer {
        request_id: Uuid,
        accepted: bool,
    },

    ImpersonateAnswer {
        grant_id: Uuid,
        accepted: bool,
    },
    ImpersonateExit,

    /// Collect logs and send them to the master. Player-initiated, so unlike
    /// `LogRequestPrompt` there is no grant to check — they pressed the button.
    SendSupportBundle {
        server_id: Option<Uuid>,
    },

    // --- Launcher updates ---
    InstallUpdate {
        version: LauncherVersion,
        modal_action: ModalAction,
    },

    UploadSkin {
        bytes: Vec<u8>,
    },

    /// Slim (Alex) or classic (Steve) arms for the skin already uploaded.
    /// Separate from `UploadSkin` because the image itself doesn't change, and
    /// asking the player for the original file again just to widen the arms
    /// would be rude.
    SetSkinModel {
        slim: bool,
    },

    RequestCapesList,
    RequestSkinPresetsList,
    SelectCape {
        cape_id: Option<Uuid>,
    },

    /// A second launcher process started and handed the request over to this one.
    FocusWindow,

    Quit,
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

    /// There is very little room next to the bar.
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

    BuildStateChanged {
        server_id: Uuid,
        state: BuildState,
    },

    ConfigState {
        memory_min_mb: u32,
        memory_max_mb: u32,
        jvm_flags: String,
        show_console_on_launch: bool,
        fullscreen: bool,
        crash_reports: bool,
        /// Whether a DSN was baked into this build; without one the toggle has
        /// nowhere to send and isn't worth showing.
        crash_reports_available: bool,
        master_url: String,
        locale: String,
        server_settings: BTreeMap<Uuid, ClientSettingsState>,
    },
    /// Translation catalog, from the master or from the local cache.
    LocaleCatalog {
        code: String,
        ftl: String,
    },
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
    /// The catalog didn't answer. The search screen needs something to end its
    /// spinner on, otherwise it sits on "searching" forever.
    CatalogFailed {
        message: String,
    },
    ModProjectLoaded {
        project: ModProjectInfo,
    },

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
    /// Packs and shaders that changed while the game was running; a resource
    /// reload picks them up. Anything the game held open is in `locked` and only
    /// lands on the next launch — the player has to be told, or they'll wait for
    /// something that already didn't happen.
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

    /// A translation key, not text. The catalog lives in the frontend, so
    /// notifications from the master follow whatever language is selected.
    AddNotification {
        key: String,
        args: BTreeMap<String, String>,
        level: NotifLevel,
    },
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

    ConnectionState {
        online: bool,
    },

    /// An admin pressed "Login as" on the site. The native dialog is a second
    /// factor: a stolen web session alone shouldn't be enough, the attacker
    /// would also need the admin's machine.
    ImpersonatePrompt {
        grant_id: Uuid,
        actor_username: String,
        target_username: String,
        reason: String,
        expires_in_secs: i64,
    },
    RemoteActionPrompt {
        action: schema::RemoteAction,
        server_id: Option<Uuid>,
        actor_username: String,
    },

    /// An admin is asking for logs; the player decides whether they go.
    LogRequestPrompt {
        request_id: Uuid,
        actor_username: String,
        reason: String,
        /// Taken without asking — the logs are already gone, the dialog is only
        /// telling them.
        forced: bool,
        /// Exactly what was or will be sent, already scrubbed.
        preview: String,
        files: Vec<(String, u64)>,
    },

    ImpersonationChanged {
        /// `None` once they're back in their own account.
        as_username: Option<String>,
    },

    OpenOrFocusMainWindow,
    CloseModal,
    Quit,
}
