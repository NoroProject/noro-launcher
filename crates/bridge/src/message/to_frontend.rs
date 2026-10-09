// Over 150 lines: one enum, a variant for each message, each documented.
//! What the backend tells the window.

use super::*;

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
        console: ConsoleSettings,
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
        /// The build's file paths, for telling "already installed" in the
        /// catalog. `None` when they haven't changed since the last message:
        /// thousands of paths used to cross over on every server open.
        installed_files: Option<Vec<String>>,
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
    /// Between "files are in place" and the game window: these take seconds
    /// on a big build, and the bar used to sit on "done" through them.
    LaunchStep {
        server_id: Uuid,
        step: LaunchStep,
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
