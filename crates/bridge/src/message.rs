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

/// One version of a catalogue project, as the version picker shows it.
///
/// A picker and not "install the newest": the newest build of a mod is
/// routinely published for a Minecraft version the build is not on yet, and
/// installing it silently is how a launcher earns a reputation for breaking
/// games.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentVersionInfo {
    pub id: String,
    pub name: String,
    pub version_number: String,
    /// `release` · `beta` · `alpha`.
    pub channel: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub downloads: u64,
    pub filename: String,
    pub size: u64,
    /// CurseForge lets an author forbid third-party downloads. Shown, not
    /// hidden — "why is this one missing" is a worse question than a greyed row.
    pub downloadable: bool,
    /// Fits the build this picker was opened for.
    pub compatible: bool,
}

/// A punishment on this account, as the player's own page shows it.
///
/// Trimmed down from what the master stores: staff ids and revocation
/// bookkeeping are not something the punished person needs, and "who banned
/// you" is already a name in `actor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunishmentView {
    pub kind: String,
    pub reason: String,
    pub actor: String,
    /// Unix seconds. Formatting is the frontend's business.
    pub created_at: i64,
    /// `None` means permanent, which is different from "no date known".
    pub expires_at: Option<i64>,
    pub active: bool,
    pub rule_code: Option<String>,
}

/// One rule from the project's rule book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleView {
    pub code: String,
    pub title: String,
    pub description: String,
    pub category: String,
    /// What breaking it costs. The point of reading the rules for most people,
    /// and the reason a rule book without them reads as a list of wishes.
    pub sanctions: Vec<SanctionView>,
}

/// One punishment a rule allows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionView {
    /// `warn` · `mute` · `ban` · `server_ban`.
    pub kind: String,
    pub label: String,
    /// Range in minutes. `None` on either side means "no bound" — and on the
    /// upper one that reads as permanent.
    pub min_minutes: Option<i64>,
    pub max_minutes: Option<i64>,
}

/// A support ticket in the list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketView {
    pub id: Uuid,
    /// Short form of the id, e.g. `a1b2c3d4`.
    ///
    /// A ticket has no number of its own on the master, and support answers
    /// with "which one?" to everything else. The first half of the uuid is
    /// stable, unique in practice and short enough to read out loud.
    pub number: String,
    pub subject: String,
    pub status: String,
    /// Staff replies the player has not opened.
    pub unread: i64,
    pub last_message_at: i64,
}

/// One message inside a ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketMessageView {
    pub author: String,
    /// Role the author held when they wrote, e.g. «Helper». Staff only.
    pub role: Option<String>,
    pub content: String,
    pub at: i64,
    /// Written by staff rather than the player.
    pub staff: bool,
}

/// A conversation in the list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmThreadView {
    pub peer_id: Uuid,
    pub peer_name: String,
    /// Where to fetch their head from. A bot has a picture of its own; a player
    /// gets one rendered from their skin.
    pub avatar_url: Option<String>,
    pub preview: String,
    pub unread: i64,
    pub last_message_at: i64,
}

/// One direct message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmMessageView {
    pub author_name: String,
    pub body: String,
    pub at: i64,
    /// Written by the player themselves.
    pub mine: bool,
}

/// An open conversation, with what the header needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmThreadOpen {
    pub peer: Uuid,
    pub peer_name: String,
    pub avatar_url: Option<String>,
    pub messages: Vec<DmMessageView>,
}

/// Frontend → Backend.
#[derive(Debug)]
pub enum MessageToBackend {
    // --- Auth ---
    /// The site owns the login methods, so this hands off to a browser.
    StartWebLogin {
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
        /// `mod` · `resourcepack` · `shader`.
        project_type: String,
        /// `relevance` · `downloads` · `follows` · `newest` · `updated`.
        sort: String,
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
    /// Whether friends on Discord see what the player is doing.
    SetDiscordRpc {
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

    // --- Notifications ---
    /// `offset` of 0 replaces the feed, anything else appends a page.
    RequestNotifications {
        offset: u32,
        unread_only: bool,
    },
    MarkNotificationRead {
        id: Uuid,
    },
    MarkAllNotificationsRead,

    // --- Personal content ---
    RequestPersonalContent {
        server_id: Uuid,
    },
    /// Versions of one project, filtered against the build it will go on.
    RequestContentVersions {
        provider: String,
        project_id: String,
        server_id: Uuid,
    },
    InstallPersonalContent {
        server_id: Uuid,
        kind: schema::personal::ContentKind,
        provider: String,
        project_id: String,
        version_id: String,
        title: String,
        icon_url: Option<String>,
    },
    RemovePersonalContent {
        server_id: Uuid,
        id: Uuid,
    },
    SetPersonalContentEnabled {
        server_id: Uuid,
        id: Uuid,
        enabled: bool,
    },

    // --- Java ---
    /// Which runtimes the master can hand out for this build.
    RequestJavaRuntimes {
        server_id: Uuid,
    },
    /// `None` goes back to the runtime the build was built against.
    SetJavaRuntime {
        server_id: Uuid,
        component: Option<String>,
    },

    // --- The player's own pages on the master ---
    RequestPunishments,
    RequestRules,
    RequestTickets,
    RequestTicket {
        id: Uuid,
    },
    OpenTicket {
        subject: String,
        content: String,
    },
    ReplyTicket {
        id: Uuid,
        content: String,
    },
    RequestDmThreads,
    RequestDmThread {
        peer: Uuid,
    },
    SendDm {
        peer: Uuid,
        body: String,
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
    /// The startup session check ended without a signed-in player: there was
    /// no stored session, or the master couldn't be reached to confirm it.
    /// Until this (or `LoginSuccess`) arrives the window shows "checking".
    SessionCheckDone,

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
        discord_rpc: bool,
        master_url: String,
        locale: String,
        server_settings: BTreeMap<Uuid, ClientSettingsState>,
        /// Physical memory of this computer, to keep the memory setting
        /// within it. `None` where it can't be read.
        system_memory_mb: Option<u32>,
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
        /// Whether the build allows adding your own content. The operator decides, not the player.
        allow_personal: bool,
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
        /// A translation key naming the cause.
        reason: String,
        /// The technical chain, for the console and support. May be empty.
        detail: String,
    },
    /// The launch was called off before it started, and whoever called it off
    /// has already told the player why: the button just goes back to normal.
    LaunchCancelled {
        server_id: Uuid,
    },

    GameStarted {
        server_id: Uuid,
    },
    GameStopped {
        server_id: Uuid,
        exit_ok: bool,
    },
    /// What the game printed since the last batch, in order. A batch rather
    /// than a line: a modded client prints thousands of lines a second at
    /// startup and at a crash, and a message per line buried the ones that say
    /// the game has exited.
    GameLog {
        server_id: Uuid,
        lines: Vec<GameLogLine>,
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

    /// A page of the feed. `offset` says whether it replaces or extends what the
    /// panel already shows.
    NotificationFeed {
        items: Vec<schema::notifications::Notification>,
        total: i64,
        offset: u32,
        unread: i64,
    },
    /// Arrived while the launcher was open.
    NotificationArrived {
        notification: Box<schema::notifications::Notification>,
        unread: i64,
        /// The master handed this client the right to raise a system toast.
        os_toast: bool,
    },
    UnreadChanged {
        unread: i64,
    },

    PersonalContent {
        server_id: Uuid,
        items: Vec<schema::personal::PersonalItem>,
    },
    ContentVersions {
        provider: String,
        project_id: String,
        versions: Vec<ContentVersionInfo>,
    },
    /// Install, removal or a version list that didn't work. Carries the
    /// master's own wording: "staff blocked this mod" is not "could not
    /// install".
    ContentActionFailed {
        message: String,
    },

    JavaRuntimes {
        server_id: Uuid,
        options: Vec<schema::java::JavaRuntimeOption>,
        /// What the build itself was built against.
        default_component: String,
        /// What this player picked, if they picked anything.
        selected: Option<String>,
    },

    PunishmentsLoaded {
        items: Vec<PunishmentView>,
    },
    RulesLoaded {
        items: Vec<RuleView>,
    },
    TicketsLoaded {
        items: Vec<TicketView>,
    },
    TicketLoaded {
        id: Uuid,
        subject: String,
        status: String,
        messages: Vec<TicketMessageView>,
    },
    DmThreadsLoaded {
        items: Vec<DmThreadView>,
    },
    DmThreadLoaded {
        thread: Box<DmThreadOpen>,
    },
    /// Arrived over the socket while the launcher was open.
    DmArrived {
        peer: Uuid,
        message: DmMessageView,
    },

    OpenOrFocusMainWindow,
    CloseModal,
    Quit,
}
