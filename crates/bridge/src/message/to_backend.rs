// Over 150 lines: one enum, a variant for each request, each documented.
//! What the window asks the backend for.

use super::*;

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
    SetConsoleSettings {
        settings: ConsoleSettings,
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
