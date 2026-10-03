//! UI state, and the handling of messages from the backend.

use bridge::{
    BackendHandle, ClientSettingsState, LoginErrorKind, MessageToBackend, MessageToFrontend,
    OptionalModInfo, SyncStage,
};
use gpui::{
    px, AppContext, Context, Entity, Image, IntoElement, ListAlignment, ListState, RenderImage,
};

use schema::{LauncherVersion, NewsItem, NotifLevel, ServerEntry, UserProfile};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

/// What the button on a catalogue card does.
///
/// Two modes rather than two buttons: the choice is a stance, not a per-mod
/// decision — either the player is kitting out their own client, or they are
/// telling staff what the build is missing. A card with both buttons makes
/// every card ask a question that was already answered.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ContentMode {
    /// Install it for this player only.
    #[default]
    Install,
    /// Ask staff to add it to the build for everyone.
    Suggest,
}

#[derive(Clone, PartialEq)]
pub enum Page {
    Login,
    Servers,
    ServerDetail(Uuid),
    ServerMods(Uuid),
    ServerModCatalog(Uuid),
    ServerSettings(Uuid),
    News,
    NewsDetail(Uuid),
    Profile,
    Settings,
    /// Punishments, tickets and the rule book — the player's standing with the
    /// project, in one place.
    Account,
    /// Conversations, and one of them when it is open.
    Messages,
}

/// Which of the account page's three lists is showing.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AccountTab {
    #[default]
    Punishments,
    Tickets,
    Rules,
}

/// What the sync panel's heading says. Kept as a state rather than as text so
/// the words follow the language.
#[derive(Clone, Debug, PartialEq)]
pub enum SyncHeading {
    Preparing,
    Stage(SyncStage),
    /// Several download stages at once; no single one is the heading.
    Downloading,
    /// Files are in place; the check and the JVM start take a few seconds.
    Launching,
    Cancelling,
}

/// Why the last sync or launch failed.
#[derive(Clone, Debug)]
pub struct SyncFailure {
    /// A translation key.
    pub reason: String,
    /// The technical chain, for whoever reads the console or a support report.
    pub detail: String,
}

/// Sync and run state for one server.
#[derive(Default, Clone)]
pub struct SyncUiState {
    pub heading: Option<SyncHeading>,
    pub detail: String,
    /// Bytes per download stage — they run in parallel and each gets its own
    /// bar. BTreeMap so the row order doesn't depend on who reported first.
    pub stages: std::collections::BTreeMap<SyncStage, (u64, u64)>,
    pub syncing: bool,
    pub failed: Option<SyncFailure>,
    pub running: bool,
    /// The launch in flight. Kept so it can be cancelled: the backend has
    /// always honoured a cancel, but nothing in the window could ask for one.
    pub launch: Option<bridge::ModalAction>,
    pub rate: crate::sync_text::Rate,
    /// The stop button was clicked once and waits for the confirming click.
    pub stop_armed: bool,
}

impl SyncUiState {
    pub fn heading_text(&self) -> String {
        match &self.heading {
            None | Some(SyncHeading::Preparing) => i18n::t("game-preparing"),
            Some(SyncHeading::Stage(stage)) => crate::sync_text::stage_label(*stage),
            Some(SyncHeading::Downloading) => i18n::t("sync-downloading"),
            Some(SyncHeading::Launching) => i18n::t("sync-launching"),
            Some(SyncHeading::Cancelling) => i18n::t("sync-cancelling"),
        }
    }

    /// The launch can still be called off: it hasn't reached the JVM yet.
    pub fn cancellable(&self) -> bool {
        self.syncing
            && self.launch.is_some()
            && !matches!(
                self.heading,
                Some(SyncHeading::Launching | SyncHeading::Cancelling)
            )
    }

    pub fn done(&self) -> u64 {
        self.stages.values().map(|(d, _)| d).sum()
    }

    pub fn total(&self) -> u64 {
        self.stages.values().map(|(_, t)| t).sum()
    }

    pub fn fraction(&self) -> f32 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (self.done() as f32 / total as f32).clamp(0.0, 1.0)
        }
    }
}

fn translate_notification(key: &str, args: &std::collections::BTreeMap<String, String>) -> String {
    if args.is_empty() {
        return i18n::t(key);
    }
    let mut fluent = i18n::FluentArgs::new();
    for (k, v) in args {
        fluent.set(k.clone(), v.clone());
    }
    i18n::t_args(key, &fluent)
}

#[derive(Clone)]
pub struct Toast {
    /// Grows with every toast. The timer has to remove its own: while it
    /// sleeps, the stack can turn over completely.
    pub id: u64,
    pub text: String,
    pub level: NotifLevel,
    /// Bumped when the same text comes again. Only the newest timer may
    /// remove the toast, which is what makes a repeat actually extend it.
    pub generation: u64,
}

impl Toast {
    /// How long the toast stays.
    ///
    /// The worse the news, the longer: "skin uploaded" is read at a glance, while
    /// the reason the game didn't start is something people read twice.
    pub fn lifetime(&self) -> std::time::Duration {
        let secs = match self.level {
            NotifLevel::Error => 9,
            NotifLevel::Warning => 7,
            _ => 4,
        };
        std::time::Duration::from_secs(secs)
    }
}

#[derive(Clone)]
pub struct UiConfig {
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub jvm_flags: String,
    pub show_console_on_launch: bool,
    pub fullscreen: bool,
    pub crash_reports: bool,
    /// Whether a DSN is baked into this build. Without one the settings row is
    /// hidden — the toggle would flip but there is nowhere to send.
    pub crash_reports_available: bool,
    pub discord_rpc: bool,
    pub master_url: String,
    pub system_memory_mb: Option<u32>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            memory_min_mb: 2048,
            memory_max_mb: 4096,
            jvm_flags: String::new(),
            show_console_on_launch: true,
            fullscreen: false,
            crash_reports: true,
            crash_reports_available: false,
            discord_rpc: true,
            master_url: String::new(),
            system_memory_mb: None,
        }
    }
}

pub use bridge::GameLogLine as LogEntry;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ProfileTab {
    #[default]
    Overview,
    Skins,
    Capes,
}

#[derive(Clone, Debug)]
pub struct SavedSkinPreset {
    pub id: String,
    pub name: String,
    /// Shared: the preset cards are drawn every frame the skin turns, and
    /// copying each preset's PNG for every one of them added up.
    pub bytes: Arc<Vec<u8>>,
}

pub struct LauncherUI {
    pub backend: BackendHandle,
    pub page: Page,
    pub profile_tab: ProfileTab,
    pub user: Option<UserProfile>,
    pub skin_image: Option<Arc<Image>>,
    /// Current preview frame. `RenderImage` and not `Image`, because it draws
    /// synchronously.
    pub skin_preview: Option<Arc<RenderImage>>,
    /// The current skin and cape, decoded once per change rather than per frame.
    pub skin_decoded: Option<(u64, Arc<crate::skin::Decoded>)>,
    /// Whether the main window has the focus. The skin preview only animates
    /// while it does.
    pub window_active: bool,
    pub skin_bytes: Option<Vec<u8>>,
    pub skin_url: Option<String>,
    /// Rotation of the figure, in degrees.
    pub skin_yaw: f32,
    /// Limb sway phase in `[0, 1)`. Its own clock, not tied to the rotation.
    pub skin_sway: f32,
    pub skin_loading: bool,
    pub skin_uploading: bool,
    /// The skin before an upload, to go back to if the upload is refused.
    pub skin_before_upload: Option<Vec<u8>>,
    pub skin_dragging: bool,
    /// Cursor x at the last drag sample, in window px.
    pub skin_drag_x: f32,
    pub skin_anim_running: bool,
    pub cape_bytes: Option<Vec<u8>>,
    pub cape_url: Option<String>,
    pub cape_loading: bool,
    pub capes: Vec<schema::CapeRow>,
    pub cape_images: std::collections::HashMap<uuid::Uuid, std::sync::Arc<gpui::Image>>,
    pub preset_images: std::collections::HashMap<String, std::sync::Arc<gpui::Image>>,
    pub custom_presets: Vec<SavedSkinPreset>,
    pub cape_selector_open: bool,
    pub build_picker_open: bool,
    pub avatar_image: Option<Arc<Image>>,
    pub avatar_loading: bool,
    /// Frame counters. Idle unless `NORO_PERF` is set.
    pub perf: crate::perf::Perf,
    /// UI language. The catalog itself lives in i18n's global state.
    pub locale: i18n::Locale,
    pub online: bool,
    /// The socket to the master went down and hasn't come back. Separate from
    /// `online`, which is also false for the moment before the first connect.
    pub connection_lost: bool,
    pub logging_in: bool,
    pub sidebar_collapsed: bool,
    pub mod_catalog_hits: Vec<bridge::CatalogHitInfo>,
    pub mod_catalog_selected: Option<bridge::CatalogHitInfo>,
    pub mod_catalog_provider: String,
    pub mod_catalog_query: String,
    pub mod_catalog_focus: Option<gpui::FocusHandle>,
    pub mod_project: Option<bridge::ModProjectInfo>,
    pub mod_detail_gallery: bool,
    pub mod_catalog_total: u32,
    pub mod_catalog_offset: u32,
    pub mod_catalog_limit: u32,
    /// Why the catalog is empty. `None` means either still searching, or
    /// nothing is wrong.
    pub mod_catalog_error: Option<String>,
    /// The skin preset being renamed: its id and the draft name. Edited in the
    /// card itself — no platform here has a system text input dialog.
    pub renaming_preset: Option<(String, String)>,
    pub rename_focus: Option<gpui::FocusHandle>,
    pub startup_checking: bool,
    pub login_error: Option<String>,
    /// The web sign-in in flight, so it can be cancelled.
    pub login_modal: Option<bridge::ModalAction>,
    pub login_mode_key: bool,
    pub login_key_input: String,
    pub login_key_focus: Option<gpui::FocusHandle>,

    pub servers: Vec<ServerEntry>,
    /// Build the player picked, per server. No entry means the current one.
    pub selected_build: std::collections::HashMap<Uuid, Option<Uuid>>,
    pub news: Vec<NewsItem>,
    pub sync: HashMap<Uuid, SyncUiState>,
    pub build_state: HashMap<Uuid, bridge::BuildState>,
    pub logs: HashMap<Uuid, std::collections::VecDeque<LogEntry>>,
    pub optional_mods: HashMap<Uuid, Vec<OptionalModInfo>>,
    pub installed_files: HashMap<Uuid, Vec<String>>,
    /// Names of what the build already ships, normalised for comparison.
    ///
    /// Computed once per manifest: a catalog card asks "is this installed?" every
    /// frame, and normalising hundreds of names each time is exactly what made the
    /// list slow.
    pub installed_keys: HashMap<Uuid, std::collections::HashSet<String>>,
    pub allow_mod_suggestions: HashMap<Uuid, bool>,
    /// Whether the build allows content of your own. Without it there is no
    /// "Mods" tab at all: a button that leads to a refusal is worse than none.
    pub allow_personal_content: HashMap<Uuid, bool>,
    pub suggested_mods: HashSet<String>,
    pub background_images: HashMap<Uuid, Arc<RenderImage>>,
    pub news_images: HashMap<Uuid, Arc<Image>>,
    /// Card text per post, made when the news arrive: parsing every post's
    /// markdown again for every frame was most of the news page's cost.
    pub news_excerpts: HashMap<Uuid, gpui::SharedString>,
    news_images_loading: HashSet<Uuid>,
    pub server_icons: HashMap<Uuid, Arc<RenderImage>>,
    /// Decoded pixels rather than the compressed file: an `Image` goes to GPUI's
    /// asset cache and is decoded there while drawing, and on a list of twenty
    /// icons that cost 311 ms a frame against 30 ms on the same screen without them.
    pub optional_mod_icons: HashMap<String, Arc<RenderImage>>,
    background_image_urls: HashMap<Uuid, String>,
    server_icon_urls: HashMap<Uuid, String>,
    background_loading: HashSet<Uuid>,
    icons_loading: HashSet<Uuid>,
    optional_mod_icons_loading: HashSet<String>,
    /// Images that aren't there: a 404, a dead link, a provider refusing.
    ///
    /// Without this list a failure looked just like "not tried yet": the request
    /// went out again every frame, and each finished attempt triggered a redraw,
    /// which meant the next frame and the next attempt. A dozen broken icons in
    /// the catalog brought the whole interface down that way.
    optional_mod_icons_failed: HashSet<String>,
    /// The same for build backgrounds and icons, by image URL.
    image_failed: HashSet<String>,

    pub update_available: Option<LauncherVersion>,
    pub updating: bool,
    /// The update download in flight; its progress is read from here.
    pub update_modal: Option<bridge::ModalAction>,
    /// A stack of toasts, oldest on top. A single one for the whole window lost
    /// the previous message: sync can report three things in a row, and only the third showed.
    pub toasts: Vec<Toast>,
    next_toast_id: u64,
    pub config: UiConfig,
    pub server_settings: HashMap<Uuid, ClientSettingsState>,
    pub server_recommendations: HashMap<Uuid, ClientSettingsState>,
    pub console_window: Option<gpui::WindowHandle<ConsoleWindow>>,
    /// The main window, to raise it when the launcher is started again.
    pub main_window: Option<gpui::AnyWindowHandle>,
    /// Closing would stop a running game or download; the window is asking.
    pub close_prompt: bool,
    /// A destructive button clicked once, waiting for the confirming click.
    pub armed_action: Option<String>,
    pub impersonate_prompt: Option<ImpersonatePrompt>,
    /// Username the launcher is currently acting as.
    pub impersonating_as: Option<String>,
    pub log_request_prompt: Option<LogRequestPrompt>,
    pub log_request_preview_open: bool,
    pub remote_action_prompt: Option<RemoteActionPrompt>,

    // ── Notifications ───────────────────────────────────────────────────────
    /// The feed, newest first. The master owns it; this is the page on screen.
    pub notifications: Vec<schema::notifications::Notification>,
    pub notifications_total: i64,
    pub unread: i64,
    pub notifications_open: bool,
    pub notifications_loading: bool,
    pub notifications_unread_only: bool,

    // ── Personal content ────────────────────────────────────────────────────
    pub personal_content: HashMap<Uuid, Vec<schema::personal::PersonalItem>>,
    /// Versions of one project, keyed by provider and project id.
    pub content_versions: HashMap<(String, String), Vec<bridge::ContentVersionInfo>>,
    /// Which project's version list is open, if any.
    pub content_picker: Option<(String, String)>,
    /// Also list versions that don't fit the build.
    ///
    /// Off by default: a popular mod has fifty versions, one or two of which fit,
    /// and finding them among rows of "not released for this build" is a search,
    /// not a choice. The list isn't hidden entirely though: seeing that the mod
    /// exists for other versions at all can matter.
    pub content_versions_all: bool,
    pub content_kind: schema::personal::ContentKind,
    pub content_mode: ContentMode,
    pub content_busy: bool,
    /// The master's own wording for the last refusal.
    pub content_error: Option<String>,
    /// Showing installed content instead of the catalogue.
    pub content_show_installed: bool,
    /// `relevance` · `downloads` · `follows` · `newest` · `updated`.
    pub content_sort: String,
    /// The build the catalog was already requested for, and whether a request is in flight.
    ///
    /// Without these two fields "the list is empty, ask" fired every frame: an
    /// empty result or an answer still on its way meant sixty requests a second,
    /// which lagged not only the launcher but the master too, since it went to
    /// Modrinth for each one.
    pub content_requested_for: Option<Uuid>,
    pub content_searching: bool,

    // ── Java runtime ────────────────────────────────────────────────────────
    pub java_options: HashMap<Uuid, Vec<schema::java::JavaRuntimeOption>>,
    pub java_selected: HashMap<Uuid, Option<String>>,
    pub java_default: HashMap<Uuid, String>,
    /// A pick is downloading a runtime on the master; it takes a while.
    pub java_busy: bool,
    pub java_picker_open: bool,
    /// The flags dialog edits a draft: the saved flags change on «Save», not
    /// on every key, and «Cancel» leaves them as they were.
    pub jvm_flags_open: bool,
    pub jvm_flags_draft: String,
    pub jvm_flags_focus: Option<gpui::FocusHandle>,

    // ── The player's own pages ──────────────────────────────────────────────
    pub account_tab: AccountTab,
    /// Which account lists were already requested.
    ///
    /// The "list is empty, ask" check sits in render, so it runs every frame. For
    /// a player with no punishments or tickets, or before the master answered,
    /// that was a request per frame, sixty a second, each with its own redraw on
    /// the reply. Hence "every list lags".
    pub account_requested: HashSet<&'static str>,
    pub punishments: Vec<bridge::PunishmentView>,
    pub rules: Vec<bridge::RuleView>,
    pub rules_query: String,
    pub rules_focus: Option<gpui::FocusHandle>,
    pub tickets: Vec<bridge::TicketView>,
    /// The ticket that is open, with its thread.
    pub ticket_open: Option<(Uuid, String, String, Vec<bridge::TicketMessageView>)>,
    pub dm_threads: Vec<bridge::DmThreadView>,
    /// The conversation list was already requested. Checking for an empty list
    /// won't do: it sits in render and asked every frame for an account with none.
    pub dm_requested: bool,
    /// The conversation that is open.
    pub dm_open: Option<bridge::DmThreadOpen>,
    /// What is being typed, in whichever of the two is open.
    pub compose: String,
    pub compose_focus: Option<gpui::FocusHandle>,
}

/// An action an admin is asking the player to take.
pub struct RemoteActionPrompt {
    pub action: schema::RemoteAction,
    pub server_id: Option<Uuid>,
    pub actor_username: String,
}

pub struct LogRequestPrompt {
    pub request_id: Uuid,
    pub actor_username: String,
    pub reason: String,
    /// Collected without asking: the logs have already gone, and the modal only
    /// says so.
    pub forced: bool,
    pub preview: String,
    pub files: Vec<(String, u64)>,
}

pub struct ImpersonatePrompt {
    pub grant_id: Uuid,
    pub target_username: String,
    pub reason: String,
    pub expires_in_secs: i64,
}
pub struct ConsoleWindow {
    pub server_id: Uuid,
    pub buffer: crate::console_model::ConsoleBuffer,
    pub list_state: ListState,
    pub status_message: String,
    pub copy_success: bool,
}

impl ConsoleWindow {
    fn new(server_id: Uuid, lines: impl IntoIterator<Item = LogEntry>) -> Self {
        let buffer = crate::console_model::ConsoleBuffer::from_lines(lines, Default::default());
        let list_state = ListState::new(buffer.visible_len(), ListAlignment::Top, px(100.));
        // Follows new lines while the reader is at the bottom; scrolling up
        // stops it, and scrolling back down picks it up again. Recreating the
        // list for every batch used to throw the reader back to the bottom.
        list_state.set_follow_mode(gpui::FollowMode::Tail);
        Self {
            server_id,
            buffer,
            list_state,
            status_message: String::new(),
            copy_success: false,
        }
    }

    /// New lines go in as a splice at the end (and one at the front for what
    /// fell off), so the reader's position holds.
    pub fn push(&mut self, lines: Vec<LogEntry>) {
        let before = self.buffer.visible_len();
        let change = self.buffer.append(lines);
        if change.removed_front > 0 {
            self.list_state.splice(0..change.removed_front, 0);
        }
        let end = before - change.removed_front;
        if change.added_back > 0 {
            self.list_state.splice(end..end, change.added_back);
        }
    }

    pub fn set_filters(&mut self, filters: crate::console_model::Filters) {
        self.buffer.set_filters(filters);
        self.list_state.reset(self.buffer.visible_len());
        self.list_state.set_follow_mode(gpui::FollowMode::Tail);
    }

    /// The console follows the server the player is looking at.
    pub fn show_server(&mut self, server_id: Uuid, lines: impl IntoIterator<Item = LogEntry>) {
        let filters = self.buffer.filters().clone();
        self.server_id = server_id;
        self.buffer = crate::console_model::ConsoleBuffer::from_lines(lines, filters);
        self.list_state.reset(self.buffer.visible_len());
        self.list_state.set_follow_mode(gpui::FollowMode::Tail);
    }

    pub fn follow(&mut self) {
        self.list_state.set_follow_mode(gpui::FollowMode::Tail);
    }
}

pub struct GlobalLauncherUI(pub Entity<LauncherUI>);
impl gpui::Global for GlobalLauncherUI {}

impl gpui::Render for ConsoleWindow {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::pages::game_console;
        game_console::console_window_body(self, cx)
    }
}

use crate::console_model::MAX_LOG_LINES;

/// Catalogue icons and avatars are drawn at a few dozen pixels.
const ICON_SIDE: u32 = 128;
/// Mod screenshots fill a wide gallery box.
const SCREENSHOT_SIDE: u32 = 480;
/// Remote pictures kept decoded at once before the cache starts over.
const REMOTE_IMAGE_CAP: usize = 256;

fn trimmed(url: Option<&String>) -> Option<&str> {
    url.map(|u| u.trim()).filter(|u| !u.is_empty())
}
const CONSOLE_WINDOW_SIZE: (f32, f32) = (800., 500.);
const CONSOLE_WINDOW_MIN_SIZE: (f32, f32) = (720., 440.);

impl LauncherUI {
    pub fn new(backend: BackendHandle) -> Self {
        // Ask right away; the answer comes once the ws connects.
        backend.send(MessageToBackend::RequestServerList);
        backend.send(MessageToBackend::RequestNews);
        Self {
            backend,
            page: Page::Login,
            profile_tab: ProfileTab::Overview,
            user: None,
            skin_image: None,
            skin_preview: None,
            skin_bytes: None,
            skin_url: None,
            skin_yaw: 0.0,
            skin_sway: 0.0,
            skin_loading: false,
            skin_uploading: false,
            skin_before_upload: None,
            skin_dragging: false,
            skin_drag_x: 0.0,
            skin_anim_running: false,
            skin_decoded: None,
            window_active: true,
            cape_bytes: None,
            cape_url: None,
            cape_loading: false,
            capes: Vec::new(),
            cape_images: std::collections::HashMap::new(),
            preset_images: std::collections::HashMap::new(),
            custom_presets: Vec::new(),
            cape_selector_open: false,
            build_picker_open: false,
            avatar_image: None,
            avatar_loading: false,
            perf: Default::default(),
            locale: i18n::Locale::default(),
            online: false,
            connection_lost: false,
            logging_in: false,
            sidebar_collapsed: false,
            mod_catalog_hits: Vec::new(),
            mod_catalog_selected: None,
            mod_catalog_provider: "modrinth".to_string(),
            mod_catalog_query: String::new(),
            mod_catalog_focus: None,
            mod_project: None,
            mod_detail_gallery: false,
            mod_catalog_total: 0,
            mod_catalog_offset: 0,
            mod_catalog_limit: 20,
            mod_catalog_error: None,
            renaming_preset: None,
            rename_focus: None,
            startup_checking: true,
            login_error: None,
            login_modal: None,
            login_mode_key: false,
            login_key_input: String::new(),
            login_key_focus: None,
            servers: Vec::new(),
            selected_build: std::collections::HashMap::new(),
            news: Vec::new(),
            sync: HashMap::new(),
            build_state: HashMap::new(),
            logs: HashMap::new(),
            optional_mods: HashMap::new(),
            installed_files: HashMap::new(),
            installed_keys: HashMap::new(),
            allow_mod_suggestions: HashMap::new(),
            allow_personal_content: HashMap::new(),
            suggested_mods: HashSet::new(),
            background_images: HashMap::new(),
            news_images: HashMap::new(),
            news_excerpts: HashMap::new(),
            news_images_loading: HashSet::new(),
            server_icons: HashMap::new(),
            optional_mod_icons: HashMap::new(),
            background_image_urls: HashMap::new(),
            server_icon_urls: HashMap::new(),
            background_loading: HashSet::new(),
            icons_loading: HashSet::new(),
            optional_mod_icons_loading: HashSet::new(),
            optional_mod_icons_failed: HashSet::new(),
            image_failed: HashSet::new(),
            update_available: None,
            impersonate_prompt: None,
            log_request_prompt: None,
            log_request_preview_open: false,
            remote_action_prompt: None,
            impersonating_as: None,
            updating: false,
            update_modal: None,
            toasts: Vec::new(),
            next_toast_id: 0,
            config: UiConfig::default(),
            server_settings: HashMap::new(),
            server_recommendations: HashMap::new(),
            console_window: None,
            main_window: None,
            close_prompt: false,
            armed_action: None,

            notifications: Vec::new(),
            notifications_total: 0,
            unread: 0,
            notifications_open: false,
            notifications_loading: false,
            notifications_unread_only: false,

            personal_content: HashMap::new(),
            content_versions: HashMap::new(),
            content_picker: None,
            content_versions_all: false,
            content_kind: schema::personal::ContentKind::Mod,
            content_mode: ContentMode::default(),
            content_busy: false,
            content_error: None,
            content_show_installed: false,
            content_sort: "relevance".to_string(),
            content_requested_for: None,
            content_searching: false,

            java_options: HashMap::new(),
            java_selected: HashMap::new(),
            java_default: HashMap::new(),
            java_busy: false,
            java_picker_open: false,
            jvm_flags_open: false,
            jvm_flags_draft: String::new(),
            jvm_flags_focus: None,

            account_tab: AccountTab::default(),
            account_requested: HashSet::new(),
            punishments: Vec::new(),
            rules: Vec::new(),
            rules_query: String::new(),
            rules_focus: None,
            tickets: Vec::new(),
            ticket_open: None,
            dm_threads: Vec::new(),
            dm_requested: false,
            dm_open: None,
            compose: String::new(),
            compose_focus: None,
        }
    }

    pub fn sync_state(&self, server_id: &Uuid) -> SyncUiState {
        self.sync.get(server_id).cloned().unwrap_or_default()
    }

    pub fn server(&self, id: &Uuid) -> Option<&ServerEntry> {
        self.servers.iter().find(|s| &s.id == id)
    }

    pub fn selected_server_id(&self) -> Option<Uuid> {
        match self.page {
            Page::ServerDetail(id)
            | Page::ServerMods(id)
            | Page::ServerModCatalog(id)
            | Page::ServerSettings(id) => Some(id),
            _ => self.servers.first().map(|s| s.id),
        }
    }

    /// What the player has for the server. `jvm_flags` here are always the
    /// player's own; the build's stay in `server_recommendations` and are added
    /// at launch whatever this says.
    pub fn server_client_settings(&self, server_id: Uuid) -> ClientSettingsState {
        if let Some(saved) = self.server_settings.get(&server_id) {
            return saved.clone();
        }
        let mut settings = self
            .server_recommendations
            .get(&server_id)
            .cloned()
            .unwrap_or_else(|| ClientSettingsState {
                memory_min_mb: self.config.memory_min_mb,
                memory_max_mb: self.config.memory_max_mb,
                jvm_flags: String::new(),
                show_console_on_launch: self.config.show_console_on_launch,
                fullscreen: self.config.fullscreen,
            });
        settings.jvm_flags = self.config.jvm_flags.clone();
        settings
    }

    pub fn has_server_client_override(&self, server_id: Uuid) -> bool {
        self.server_settings.contains_key(&server_id)
    }

    pub fn ensure_background_loaded(
        &mut self,
        server_id: Uuid,
        url: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(url) = url.filter(|u| !u.trim().is_empty()) else {
            return;
        };
        if self.image_failed.contains(&url)
            || (self.background_image_urls.get(&server_id) == Some(&url)
                && (self.background_images.contains_key(&server_id)
                    || self.background_loading.contains(&server_id)))
        {
            return;
        }

        // The old picture stays up until the new one is in; replacing it is
        // also what hands its texture back to GPUI below. Removing it first
        // meant the replacement never found anything to release.
        self.background_image_urls.insert(server_id, url.clone());
        self.background_loading.insert(server_id);
        cx.spawn(async move |this, cx| {
            let expected_url = url.clone();
            let result = crate::image_loader::load_render_image_capped(url, 1600).await;
            let _ = this.update(cx, |state, cx| {
                state.background_loading.remove(&server_id);
                if state.background_image_urls.get(&server_id) != Some(&expected_url) {
                    return;
                }
                match result {
                    Ok(image) => {
                        // Hand the old texture back to GPUI: the atlas keeps every
                        // `RenderImage` by id and never evicts one on its own, so
                        // changing the background would leave megabytes behind.
                        if let Some(stale) = state.background_images.insert(server_id, image) {
                            if Arc::strong_count(&stale) == 1 {
                                cx.drop_image(stale, None);
                            }
                        }
                    }
                    Err(err) => {
                        // Once per URL. A failure used not to be remembered, and
                        // the next frame downloaded again, with a new error toast
                        // for every attempt.
                        state.image_failed.insert(expected_url);
                        let mut args = i18n::FluentArgs::new();
                        args.set("reason", err.to_string());
                        state.notify_toast(
                            i18n::t_args("error-background-failed", &args),
                            NotifLevel::Warning,
                            cx,
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn ensure_icon_loaded(
        &mut self,
        server_id: Uuid,
        url: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(url) = url.filter(|u| !u.trim().is_empty()) else {
            return;
        };
        if self.image_failed.contains(&url)
            || (self.server_icon_urls.get(&server_id) == Some(&url)
                && (self.server_icons.contains_key(&server_id)
                    || self.icons_loading.contains(&server_id)))
        {
            return;
        }
        self.server_icon_urls.insert(server_id, url.clone());
        self.icons_loading.insert(server_id);
        cx.spawn(async move |this, cx| {
            let expected_url = url.clone();
            let result = crate::image_loader::load_render_image_capped(url, 256).await;
            let _ = this.update(cx, |state, cx| {
                state.icons_loading.remove(&server_id);
                if state.server_icon_urls.get(&server_id) != Some(&expected_url) {
                    return;
                }
                match result {
                    Ok(image) => {
                        if let Some(stale) = state.server_icons.insert(server_id, image) {
                            if Arc::strong_count(&stale) == 1 {
                                cx.drop_image(stale, None);
                            }
                        }
                    }
                    Err(_) => {
                        state.image_failed.insert(expected_url);
                    }
                };
                cx.notify();
            });
        })
        .detach();
    }

    /// How many images never loaded. For the overlay, to tell "no icons"
    /// from "icons aren't arriving".
    pub fn failed_image_count(&self) -> usize {
        self.optional_mod_icons_failed.len() + self.image_failed.len()
    }

    pub fn ensure_optional_mod_icon_loaded(&mut self, url: Option<String>, cx: &mut Context<Self>) {
        self.ensure_remote_image_loaded(url, ICON_SIDE, cx);
    }

    /// Screenshots are shown much larger than icons; decoded at icon size they
    /// came out blurry.
    pub fn ensure_screenshot_loaded(&mut self, url: Option<String>, cx: &mut Context<Self>) {
        self.ensure_remote_image_loaded(url, SCREENSHOT_SIDE, cx);
    }

    fn ensure_remote_image_loaded(
        &mut self,
        url: Option<String>,
        max_side: u32,
        cx: &mut Context<Self>,
    ) {
        let Some(url) = url.filter(|u| !u.trim().is_empty()) else {
            return;
        };
        if self.optional_mod_icons.contains_key(&url)
            || self.optional_mod_icons_loading.contains(&url)
            || self.optional_mod_icons_failed.contains(&url)
        {
            return;
        }
        self.optional_mod_icons_loading.insert(url.clone());
        cx.spawn(async move |this, cx| {
            let result = crate::image_loader::load_render_image_capped(url.clone(), max_side).await;
            let _ = this.update(cx, |state, cx| {
                state.optional_mod_icons_loading.remove(&url);
                match result {
                    Ok(image) => {
                        // Catalogue pages, avatars and screenshots all land
                        // here, and nothing ever left. Past the cap the lot is
                        // released; what is still on screen loads again.
                        if state.optional_mod_icons.len() >= REMOTE_IMAGE_CAP {
                            for (_, stale) in state.optional_mod_icons.drain() {
                                cx.drop_image(stale, None);
                            }
                        }
                        state.optional_mod_icons.insert(url, image);
                        // Redraw only when there is something to show: otherwise
                        // a failure triggers the very frame that repeats it.
                        cx.notify();
                    }
                    Err(e) => {
                        tracing::debug!(url = %url, error = %e, "icon did not load");
                        state.optional_mod_icons_failed.insert(url);
                    }
                }
            });
        })
        .detach();
    }

    fn replace_servers(&mut self, servers: Vec<ServerEntry>, cx: &mut Context<Self>) {
        let next_ids: HashSet<_> = servers.iter().map(|s| s.id).collect();
        let old_ids: Vec<_> = self.servers.iter().map(|s| s.id).collect();

        for id in old_ids {
            if !next_ids.contains(&id) {
                self.clear_background(id, cx);
                self.clear_icon(id, cx);
            }
        }

        for server in &servers {
            if trimmed(server.background_url.as_ref()).is_none() {
                self.clear_background(server.id, cx);
            }
            if trimmed(server.icon_url.as_ref()).is_none() {
                self.clear_icon(server.id, cx);
            }
        }
        // A changed address needs nothing here: `ensure_*_loaded` sees the new
        // URL on the next frame, loads it, and releases the old picture when
        // the new one replaces it.

        self.servers = servers;
    }

    /// Every removal hands the texture back to GPUI, whose atlas never evicts
    /// anything on its own.
    fn clear_background(&mut self, server_id: Uuid, cx: &mut Context<Self>) {
        if let Some(image) = self.background_images.remove(&server_id) {
            cx.drop_image(image, None);
        }
        self.background_loading.remove(&server_id);
        self.background_image_urls.remove(&server_id);
    }

    fn clear_icon(&mut self, server_id: Uuid, cx: &mut Context<Self>) {
        if let Some(image) = self.server_icons.remove(&server_id) {
            cx.drop_image(image, None);
        }
        self.icons_loading.remove(&server_id);
        self.server_icon_urls.remove(&server_id);
    }

    pub fn load_preset_renders(&mut self, cx: &mut Context<Self>) {
        if self.preset_images.contains_key("steve") {
            return;
        }
        let master_url = self.config.master_url.clone();
        let presets = [
            "steve", "alex", "ari", "zuri", "efe", "makena", "kai", "sunny", "noor",
        ];
        for preset in presets {
            let name = preset.to_string();
            let url = format!(
                "{}/api/textures/renders/bust?preset={}&scale=8&yaw=-25&pitch=12",
                master_url.trim_end_matches('/'),
                name
            );
            cx.spawn(async move |this, cx| {
                if let Ok(img) = crate::image_loader::load_image_from_url(url).await {
                    let _ = this.update(cx, |this, cx| {
                        this.preset_images.insert(name, img);
                        cx.notify();
                    });
                }
            })
            .detach();
        }
    }

    pub fn on_message(&mut self, msg: MessageToFrontend, cx: &mut Context<Self>) {
        match msg {
            MessageToFrontend::LoginSuccess { user } => {
                // The bell counter has to be right before the panel is opened:
                // otherwise anything unread that arrived while offline would
                // never show.
                self.backend.send(MessageToBackend::RequestNotifications {
                    offset: 0,
                    unread_only: false,
                });
                self.user = Some(user);
                self.load_user_skin(cx);
                self.logging_in = false;
                self.login_modal = None;
                self.startup_checking = false;
                self.login_error = None;
                self.backend.send(MessageToBackend::RequestCapesList);
                self.backend.send(MessageToBackend::RequestSkinPresetsList);
                if self.page == Page::Login {
                    self.page = Page::Servers;
                }
            }
            MessageToFrontend::LoginFailed { kind } => {
                self.logging_in = false;
                self.login_modal = None;
                self.startup_checking = false;
                self.login_error = Some(match kind {
                    LoginErrorKind::Cancelled => i18n::t("error-sign-in-cancelled"),
                    // `r` is a translation key from the master, not text.
                    LoginErrorKind::Rejected(r) => i18n::t(&r),
                    LoginErrorKind::Network(e) => {
                        let mut args = i18n::FluentArgs::new();
                        args.set("reason", e);
                        i18n::t_args("error-network", &args)
                    }
                });
            }
            MessageToFrontend::LoggedOut => {
                self.clear_account_data();
                self.user = None;
                self.reset_skin_preview(cx);
                self.skin_url = None;
                self.skin_loading = false;
                self.skin_uploading = false;
                self.skin_dragging = false;
                self.skin_anim_running = false;
                self.cape_bytes = None;
                self.cape_url = None;
                self.cape_loading = false;
                self.avatar_image = None;
                self.avatar_loading = false;
                self.logging_in = false;
                self.startup_checking = false;
                self.page = Page::Login;
            }
            MessageToFrontend::ServerList { servers } => self.replace_servers(servers, cx),
            MessageToFrontend::NewsUpdated { items } => {
                let ids: HashSet<Uuid> = items.iter().map(|n| n.id).collect();
                self.news_images.retain(|id, _| ids.contains(id));
                self.news_excerpts = items
                    .iter()
                    .map(|n| (n.id, crate::pages::plain_excerpt(&n.body, 240).into()))
                    .collect();
                self.news = items;
            }
            MessageToFrontend::ConfigState {
                memory_min_mb,
                memory_max_mb,
                jvm_flags,
                show_console_on_launch,
                fullscreen,
                crash_reports,
                crash_reports_available,
                discord_rpc,
                master_url,
                locale,
                server_settings,
                system_memory_mb,
            } => {
                if let Some(loc) = i18n::Locale::from_code(&locale) {
                    self.locale = loc;
                    i18n::set_locale(loc);
                }
                self.config = UiConfig {
                    memory_min_mb,
                    memory_max_mb,
                    jvm_flags,
                    show_console_on_launch,
                    fullscreen,
                    crash_reports,
                    crash_reports_available,
                    discord_rpc,
                    master_url,
                    system_memory_mb,
                };
                self.server_settings = server_settings.into_iter().collect();
                self.load_preset_renders(cx);
            }
            MessageToFrontend::SessionCheckDone => {
                self.startup_checking = false;
            }
            MessageToFrontend::LocaleCatalog { code, ftl } => {
                // The master's catalog overrides the built-in one; a broken one
                // is ignored.
                if let Some(loc) = i18n::Locale::from_code(&code) {
                    if loc == self.locale && !i18n::install_catalog(loc, &ftl) {
                        tracing::warn!("translation catalog from the master did not parse");
                    }
                }
            }

            MessageToFrontend::OptionalMods {
                server_id,
                mods,
                allow_suggestions,
                allow_personal,
                installed_files,
            } => {
                self.optional_mods.insert(server_id, mods);
                self.allow_mod_suggestions
                    .insert(server_id, allow_suggestions);
                self.allow_personal_content
                    .insert(server_id, allow_personal);
                let mut keys: std::collections::HashSet<String> = self
                    .optional_mods
                    .get(&server_id)
                    .map(|mods| {
                        mods.iter()
                            .map(|m| crate::pages::normalized_mod_name(&m.name))
                            .filter(|k| !k.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();
                keys.extend(
                    installed_files
                        .iter()
                        .filter_map(|f| {
                            std::path::Path::new(f)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .map(crate::pages::normalized_mod_name)
                        })
                        .filter(|k| !k.is_empty()),
                );
                self.installed_keys.insert(server_id, keys);
                self.installed_files.insert(server_id, installed_files);
            }
            MessageToFrontend::ServerClientRecommendation {
                server_id,
                settings,
            } => {
                self.server_recommendations.insert(server_id, settings);
            }
            MessageToFrontend::CatalogSearchResults {
                hits,
                total,
                offset,
                limit,
            } => {
                self.mod_catalog_hits = hits;
                self.mod_catalog_total = total;
                self.mod_catalog_offset = offset;
                self.mod_catalog_limit = limit;
                self.mod_catalog_error = None;
                self.content_searching = false;
            }
            MessageToFrontend::CatalogFailed { message } => {
                self.content_searching = false;
                self.mod_catalog_error = Some(message);
            }
            MessageToFrontend::ModProjectLoaded { project } => {
                // The reply can land after the player has moved to another mod.
                let still_open = self
                    .mod_catalog_selected
                    .as_ref()
                    .is_some_and(|s| s.project_id == project.project_id);
                if still_open {
                    self.mod_project = Some(project);
                }
            }
            MessageToFrontend::SyncProgress {
                server_id,
                stage,
                done,
                total,
                file,
            } => {
                let s = self.sync.entry(server_id).or_default();
                s.syncing = stage != SyncStage::Done;
                let cancelling = s.heading == Some(SyncHeading::Cancelling);
                if stage.is_download() {
                    s.stages.insert(stage, (done, total));
                    s.rate.record(s.done());
                    if !cancelling {
                        s.heading = Some(SyncHeading::Downloading);
                    }
                } else {
                    // Checking files opens a new pass; the bars from the last
                    // run don't belong to it.
                    if stage == SyncStage::CheckingFiles && done == 0 {
                        s.stages.clear();
                        s.rate.clear();
                    }
                    if !cancelling {
                        s.heading = Some(SyncHeading::Stage(stage));
                    }
                }
                if !file.is_empty() {
                    s.detail = file;
                }
                s.failed = None;
            }
            MessageToFrontend::SyncComplete { server_id } => {
                // Not playable yet: the files get checked and the JVM started,
                // which for a big build takes seconds. The button stays busy
                // until GameStarted or SyncFailed — a second click here started
                // a second game in the same folder.
                let s = self.sync.entry(server_id).or_default();
                s.heading = Some(SyncHeading::Launching);
            }
            MessageToFrontend::LaunchCancelled { server_id } => {
                let s = self.sync.entry(server_id).or_default();
                s.syncing = false;
                s.failed = None;
                s.launch = None;
                s.heading = None;
                s.stages.clear();
                s.rate.clear();
            }
            MessageToFrontend::LiveSynced {
                server_id: _,
                updated,
                locked,
            } => {
                // New packs arrived while the game is running. Nothing shows
                // until the client reloads its resources, and that's the
                // player's call: mid-fight it isn't welcome. Said in a toast:
                // the sync panel this used to go to is hidden while playing.
                let text = if locked.is_empty() {
                    i18n::t_count("sync-live-updated", updated.len() as i64)
                } else {
                    let mut args = i18n::FluentArgs::new();
                    args.set("count", updated.len() as i64);
                    args.set("locked", locked.len() as i64);
                    i18n::t_args("sync-live-partial", &args)
                };
                self.notify_toast(text, NotifLevel::Info, cx);
            }
            MessageToFrontend::SyncFailed {
                server_id,
                reason,
                detail,
            } => {
                let s = self.sync.entry(server_id).or_default();
                s.syncing = false;
                s.launch = None;
                s.heading = None;
                s.rate.clear();
                self.notify_toast(i18n::t(&reason), NotifLevel::Error, cx);
                // The technical chain goes where people look when a toast isn't
                // enough: the console, and from there a support report.
                if !detail.is_empty() {
                    let logs = self.logs.entry(server_id).or_default();
                    logs.push_back(LogEntry {
                        timestamp: chrono::Utc::now().timestamp_millis(),
                        level: bridge::GameLogLevel::Error,
                        text: format!("[launcher] {detail}"),
                    });
                }
                self.sync.entry(server_id).or_default().failed =
                    Some(SyncFailure { reason, detail });
            }
            MessageToFrontend::GameStarted { server_id } => {
                let s = self.sync.entry(server_id).or_default();
                s.running = true;
                s.syncing = false;
                s.launch = None;
                s.heading = None;
                if self
                    .server_client_settings(server_id)
                    .show_console_on_launch
                {
                    self.open_console(server_id, cx);
                }
            }
            MessageToFrontend::GameStopped { server_id, exit_ok } => {
                let s = self.sync.entry(server_id).or_default();
                s.running = false;
                if !exit_ok {
                    if self
                        .server_client_settings(server_id)
                        .show_console_on_launch
                    {
                        self.open_console(server_id, cx);
                    }
                    self.notify_toast(i18n::t("error-game-exited"), NotifLevel::Warning, cx);
                }
            }
            MessageToFrontend::GameLog { server_id, lines } => {
                let logs = self.logs.entry(server_id).or_default();
                logs.extend(lines.iter().cloned());
                while logs.len() > MAX_LOG_LINES {
                    logs.pop_front();
                }

                if let Some(handle) = &self.console_window {
                    let _ = handle.update(cx, |view, _, cx| {
                        if view.server_id == server_id {
                            view.push(lines);
                            cx.notify();
                        }
                    });
                }
                // Nothing in the main window shows the log; redrawing it for
                // every batch kept the launcher busy for as long as the game
                // was writing.
                return;
            }
            MessageToFrontend::BuildStateChanged { server_id, state } => {
                self.build_state.insert(server_id, state);
            }
            MessageToFrontend::LauncherUpdateAvailable { version } => {
                self.update_available = Some(version);
            }
            MessageToFrontend::AddNotification { key, args, level } => {
                self.notify_toast(translate_notification(&key, &args), level, cx);
            }
            MessageToFrontend::ImpersonatePrompt {
                grant_id,
                target_username,
                reason,
                expires_in_secs,
                ..
            } => {
                self.impersonate_prompt = Some(ImpersonatePrompt {
                    grant_id,
                    target_username,
                    reason,
                    expires_in_secs,
                });
            }
            MessageToFrontend::LogRequestPrompt {
                request_id,
                actor_username,
                reason,
                forced,
                preview,
                files,
            } => {
                self.log_request_preview_open = false;
                self.log_request_prompt = Some(LogRequestPrompt {
                    request_id,
                    actor_username,
                    reason,
                    forced,
                    preview,
                    files,
                });
            }
            MessageToFrontend::RemoteActionPrompt {
                action,
                server_id,
                actor_username,
            } => {
                self.remote_action_prompt = Some(RemoteActionPrompt {
                    action,
                    server_id,
                    actor_username,
                });
            }
            MessageToFrontend::ImpersonationChanged { as_username } => {
                self.impersonate_prompt = None;
                self.impersonating_as = as_username;
                // Another account now: what was loaded belongs to the last one.
                self.clear_account_data();
            }
            MessageToFrontend::SkinUploadFailed => {
                self.skin_uploading = false;
                self.skin_bytes = self.skin_before_upload.take();
            }
            MessageToFrontend::PermissionsUpdated { user } => {
                self.user = Some(user);
                self.load_user_skin(cx);
            }
            MessageToFrontend::CapesList { capes } => {
                self.capes = capes.clone();
                let master_url = self.config.master_url.clone();
                for cape in capes {
                    let id = cape.id;
                    let render_url = format!(
                        "{}/api/textures/renders/cape?url={}&scale=10",
                        master_url.trim_end_matches('/'),
                        cape.url
                    );
                    cx.spawn(async move |this, cx| {
                        if let Ok(img) = crate::image_loader::load_image_from_url(render_url).await
                        {
                            let _ = this.update(cx, |this, cx| {
                                this.cape_images.insert(id, img);
                                cx.notify();
                            });
                        }
                    })
                    .detach();
                }
            }
            MessageToFrontend::SkinPresetsList { presets } => {
                self.custom_presets.clear();
                let master_url = self.config.master_url.clone();
                for p in presets {
                    let id = p.id;
                    let name = p.name;
                    let url = p.skin_url;
                    let preset_struct = SavedSkinPreset {
                        id: id.clone(),
                        name: name.clone(),
                        bytes: Arc::default(),
                    };
                    self.custom_presets.push(preset_struct);

                    let url_bytes = url.clone();
                    let id_bytes = id.clone();
                    cx.spawn(async move |this, cx| {
                        if let Ok((_, bytes)) =
                            crate::image_loader::load_image_and_bytes(url_bytes).await
                        {
                            let _ = this.update(cx, |this, cx| {
                                if let Some(found) =
                                    this.custom_presets.iter_mut().find(|cp| cp.id == id_bytes)
                                {
                                    found.bytes = Arc::new(bytes);
                                }
                                cx.notify();
                            });
                        }
                    })
                    .detach();

                    let render_url = format!(
                        "{}/api/textures/renders/bust?url={}&scale=8&yaw=-25&pitch=12",
                        master_url.trim_end_matches('/'),
                        urlencoding::encode(&url)
                    );
                    let id_render = id.clone();
                    cx.spawn(async move |this, cx| {
                        if let Ok(img) = crate::image_loader::load_image_from_url(render_url).await
                        {
                            let _ = this.update(cx, |this, cx| {
                                this.preset_images.insert(id_render, img);
                                cx.notify();
                            });
                        }
                    })
                    .detach();
                }
            }
            MessageToFrontend::ConnectionState { online } => {
                let was_lost = self.connection_lost;
                self.online = online;
                self.connection_lost = !online;
                // Images that failed while offline are worth one more try:
                // started without a network, the launcher otherwise showed no
                // icons or backgrounds for the whole session.
                if online && was_lost {
                    self.image_failed.clear();
                    self.optional_mod_icons_failed.clear();
                }
            }
            MessageToFrontend::NotificationFeed {
                items,
                total,
                offset,
                unread,
            } => {
                // Offset 0 is a refresh, anything else a further page. Appending
                // both ways would double the feed every time the panel reopens.
                if offset == 0 {
                    self.notifications = items;
                } else {
                    self.notifications.extend(items);
                }
                self.notifications_total = total;
                self.unread = unread;
                self.notifications_loading = false;
            }
            MessageToFrontend::NotificationArrived {
                notification,
                unread,
                ..
            } => {
                // Newest first, and a repeat of something already listed
                // replaces it: the master collapses repeats into one row with a
                // counter, and keeping the old copy would show both.
                self.notifications.retain(|n| n.id != notification.id);
                self.notifications.insert(0, *notification);
                self.unread = unread;
            }
            MessageToFrontend::UnreadChanged { unread } => {
                self.unread = unread;
            }

            MessageToFrontend::PersonalContent { server_id, items } => {
                self.personal_content.insert(server_id, items);
                self.content_busy = false;
                self.content_error = None;
            }
            MessageToFrontend::ContentVersions {
                provider,
                project_id,
                versions,
            } => {
                self.content_versions
                    .insert((provider, project_id), versions);
                self.content_busy = false;
            }
            MessageToFrontend::ContentActionFailed { message } => {
                self.content_busy = false;
                self.java_busy = false;
                self.content_error = Some(message);
            }

            MessageToFrontend::JavaRuntimes {
                server_id,
                options,
                default_component,
                selected,
            } => {
                self.java_options.insert(server_id, options);
                self.java_default.insert(server_id, default_component);
                self.java_selected.insert(server_id, selected);
                self.java_busy = false;
            }

            MessageToFrontend::PunishmentsLoaded { items } => self.punishments = items,
            MessageToFrontend::RulesLoaded { items } => self.rules = items,
            MessageToFrontend::TicketsLoaded { items } => self.tickets = items,
            MessageToFrontend::TicketLoaded { id, messages, .. } => {
                // Subject and status come from the list: the messages endpoint returns
                // only the messages, and putting empty strings here would erase the
                // title of the open ticket.
                let (subject, status) = self
                    .tickets
                    .iter()
                    .find(|t| t.id == id)
                    .map(|t| (t.subject.clone(), t.status.clone()))
                    .unwrap_or_default();
                self.ticket_open = Some((id, subject, status, messages));
                self.compose.clear();
            }
            MessageToFrontend::DmThreadsLoaded { items } => self.dm_threads = items,
            MessageToFrontend::DmThreadLoaded { thread } => {
                self.dm_open = Some(*thread);
                self.compose.clear();
            }
            MessageToFrontend::DmArrived { peer, message } => {
                // Only into the conversation that is open. The list of threads
                // is refetched instead: its previews and unread counts are the
                // master's arithmetic, not ours.
                if let Some(open) = self.dm_open.as_mut() {
                    if open.peer == peer {
                        open.messages.push(message);
                    }
                }
                self.backend.send(MessageToBackend::RequestDmThreads);
            }

            MessageToFrontend::OpenOrFocusMainWindow => {
                // A second launch hands over to this one. Deferred: raising the
                // window updates it, and this runs inside an update of its view.
                if let Some(handle) = self.main_window {
                    cx.defer(move |cx| {
                        let _ = handle.update(cx, |_, window, _| window.activate_window());
                        cx.activate(true);
                    });
                }
            }
            MessageToFrontend::CloseModal => {
                self.logging_in = false;
            }
            MessageToFrontend::Quit => {
                cx.quit();
            }
        }
        cx.notify();
    }

    /// Show a toast and remove it on a timer.
    ///
    /// The timer sleeps in the background and wakes the window once, to remove it.
    /// Counting the time left in render would mean redrawing for all those seconds,
    /// burning frames on a fading label.
    pub fn notify_toast(&mut self, text: String, level: NotifLevel, cx: &mut Context<Self>) {
        // The same text again doesn't become a second toast: two identical
        // lines side by side look like a glitch. The one up stays longer.
        let (id, generation, lifetime) =
            if let Some(existing) = self.toasts.iter_mut().find(|t| t.text == text) {
                existing.generation += 1;
                (existing.id, existing.generation, existing.lifetime())
            } else {
                let id = self.next_toast_id;
                self.next_toast_id += 1;
                let toast = Toast {
                    id,
                    text,
                    level,
                    generation: 0,
                };
                let lifetime = toast.lifetime();
                self.toasts.push(toast);
                // More than four is a wall, not messages; the oldest goes early.
                if self.toasts.len() > 4 {
                    self.toasts.remove(0);
                }
                (id, 0, lifetime)
            };

        // The timer sleeps in the background and wakes the window once, to
        // remove the toast. Counting down in render would redraw every frame
        // for a fading line of text.
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            executor.timer(lifetime).await;
            let _ = this.update(cx, |state, cx| {
                let current = state
                    .toasts
                    .iter()
                    .any(|t| t.id == id && t.generation == generation);
                if current {
                    state.dismiss_toast(id);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub fn dismiss_toast(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }

    // --- Actions from the UI ---

    /// Search the catalogue with whatever the browser's filters currently say.
    ///
    /// One place, because every control on that screen — the text field, the
    /// provider buttons, the content type, the sort, paging — asks the same
    /// question with one field changed. Eight copies of the message is eight
    /// places to forget a new filter in.
    pub fn search_content(&mut self, server_id: Uuid, offset: u32) {
        let server = self.server(&server_id);
        let mc_version = server.map(|s| s.mc_version.clone());
        let loader = server.map(|s| s.modloader.as_str().to_string());

        self.mod_catalog_offset = offset;
        self.mod_catalog_error = None;
        self.content_searching = true;
        self.content_requested_for = Some(server_id);
        self.backend.send(MessageToBackend::SearchCatalog {
            query: self.mod_catalog_query.trim().to_string(),
            provider: self.mod_catalog_provider.clone(),
            mc_version,
            loader,
            project_type: self.content_kind.project_type().to_string(),
            sort: self.content_sort.clone(),
            offset,
        });
    }

    /// Sign in through the website: every provider we support lives there.
    pub fn start_login(&mut self) {
        self.logging_in = true;
        self.login_error = None;
        let modal = bridge::ModalAction::new("Website sign in");
        self.login_modal = Some(modal.clone());
        self.backend.send(MessageToBackend::StartWebLogin {
            modal_action: modal,
        });
    }

    /// Everything that belongs to the signed-in account. Left in place, the
    /// next account — another player on the same computer, or an admin
    /// entering someone's account — saw the previous one's tickets, messages
    /// and punishments, and the "already loaded" flags kept them from being
    /// fetched again.
    fn clear_account_data(&mut self) {
        self.account_requested.clear();
        self.punishments.clear();
        self.tickets.clear();
        self.ticket_open = None;
        self.dm_threads.clear();
        self.dm_open = None;
        self.dm_requested = false;
        self.compose.clear();
        self.notifications.clear();
        self.notifications_total = 0;
        self.unread = 0;
        self.capes.clear();
        self.cape_images.clear();
        let custom: HashSet<String> = self.custom_presets.iter().map(|p| p.id.clone()).collect();
        self.preset_images.retain(|id, _| !custom.contains(id));
        self.custom_presets.clear();
        self.personal_content.clear();
        self.suggested_mods.clear();
    }

    /// The most the memory steppers go to: this computer's RAM less a gigabyte
    /// for the system, or the old fixed cap where the RAM isn't known. The
    /// steppers used to go to 64 GB on any machine.
    pub fn memory_ceiling_mb(&self) -> u32 {
        self.config
            .system_memory_mb
            .map(|m| m.saturating_sub(1024).max(1024))
            .unwrap_or(65536)
            .min(65536)
    }

    /// The game's share past three quarters of the RAM leaves the system and
    /// the launcher swapping, which looks like the game lagging.
    pub fn memory_warning(&self, max_mb: u32) -> Option<String> {
        let total = self.config.system_memory_mb?;
        if (max_mb as u64) * 4 <= (total as u64) * 3 {
            return None;
        }
        let mut args = i18n::FluentArgs::new();
        args.set("total", format!("{:.0}", total as f64 / 1024.0));
        Some(i18n::t_args("settings-memory-too-much", &args))
    }

    /// Closes whatever sits on top; false when nothing was open.
    pub fn close_top_overlay(&mut self) -> bool {
        if std::mem::take(&mut self.close_prompt) {
            return true;
        }
        if std::mem::take(&mut self.jvm_flags_open) {
            return true;
        }
        if std::mem::take(&mut self.java_picker_open) {
            return true;
        }
        if self.content_picker.take().is_some() {
            return true;
        }
        if std::mem::take(&mut self.build_picker_open) {
            return true;
        }
        if std::mem::take(&mut self.notifications_open) {
            return true;
        }
        if std::mem::take(&mut self.log_request_preview_open) {
            return true;
        }
        if self.mod_catalog_selected.take().is_some() {
            self.mod_project = None;
            return true;
        }
        false
    }

    /// Whether the window may close now. With a game running or a download in
    /// flight it may not: closing would stop them, so the window asks first.
    pub fn request_close(&mut self) -> bool {
        let busy = self.sync.values().any(|s| s.running || s.syncing);
        if busy {
            self.close_prompt = true;
        }
        !busy
    }

    /// The sign-in task polls the flag and answers with a cancelled login.
    pub fn cancel_login(&mut self) {
        if let Some(modal) = self.login_modal.take() {
            modal.cancel();
        }
        self.logging_in = false;
    }

    pub fn logout(&mut self) {
        self.backend.send(MessageToBackend::Logout);
    }

    pub fn upload_skin(&mut self, bytes: Vec<u8>) {
        if self.skin_uploading {
            return;
        }
        // Shown right away; put back if the master turns it down, or the
        // preview went on showing a skin that was never accepted.
        self.skin_before_upload = self.skin_bytes.replace(bytes.clone());
        self.skin_uploading = true;
        self.backend.send(MessageToBackend::UploadSkin { bytes });
    }

    /// Switch the skin model. The image stays as it is and only the arm width
    /// changes; the profile comes back the same way it does after an upload.
    pub fn set_skin_model(&mut self, slim: bool, _cx: &mut Context<Self>) {
        if self.skin_uploading || self.user.as_ref().is_none_or(|u| u.skin_slim == slim) {
            return;
        }
        self.backend.send(MessageToBackend::SetSkinModel { slim });
    }

    pub fn open_news(&mut self, id: Uuid, cx: &mut Context<Self>) {
        self.page = Page::NewsDetail(id);
        self.load_news_image(id, cx);
    }

    /// News images load lazily: the list doesn't need them, and there can be a
    /// lot of news.
    pub fn load_news_image(&mut self, id: Uuid, cx: &mut Context<Self>) {
        if self.news_images.contains_key(&id) || self.news_images_loading.contains(&id) {
            return;
        }
        let Some(url) = self
            .news
            .iter()
            .find(|n| n.id == id)
            .and_then(|n| n.preview_img_url.clone())
            .filter(|u| !u.trim().is_empty())
        else {
            return;
        };

        self.news_images_loading.insert(id);
        cx.spawn(async move |this, cx| {
            let result = crate::image_loader::load_image_capped(url, 1200).await;
            let _ = this.update(cx, |state, cx| {
                state.news_images_loading.remove(&id);
                if let Ok(image) = result {
                    state.news_images.insert(id, image);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn open_server(&mut self, id: Uuid) {
        self.page = Page::ServerDetail(id);
        self.backend
            .send(MessageToBackend::OpenServer { server_id: id });
    }

    pub fn launch(&mut self, id: Uuid) {
        let modal = bridge::ModalAction::new("Launch");
        let s = self.sync.entry(id).or_default();
        s.syncing = true;
        s.failed = None;
        s.heading = Some(SyncHeading::Preparing);
        s.launch = Some(modal.clone());
        self.backend.send(MessageToBackend::LaunchServer {
            server_id: id,
            modal_action: modal,
        });
    }

    /// Asks the running sync to stop. It stops between chunks and reports back
    /// with `LaunchCancelled`; the button says "cancelling" until then, so a
    /// new launch can't start under the old one.
    pub fn cancel_launch(&mut self, id: Uuid) {
        if let Some(s) = self.sync.get_mut(&id) {
            if let Some(modal) = &s.launch {
                modal.cancel();
                s.heading = Some(SyncHeading::Cancelling);
            }
        }
    }

    /// Two clicks for anything that can't be undone: the first arms the button
    /// for a few seconds, the second goes through. True on the second.
    pub fn confirm_or_arm(&mut self, key: String, cx: &mut Context<Self>) -> bool {
        if self.armed_action.as_deref() == Some(key.as_str()) {
            self.armed_action = None;
            return true;
        }
        self.armed_action = Some(key.clone());
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            executor.timer(std::time::Duration::from_secs(4)).await;
            let _ = this.update(cx, |ui, cx| {
                if ui.armed_action.as_deref() == Some(key.as_str()) {
                    ui.armed_action = None;
                    cx.notify();
                }
            });
        })
        .detach();
        false
    }

    pub fn is_armed(&self, key: &str) -> bool {
        self.armed_action.as_deref() == Some(key)
    }

    /// First click arms, second click stops; the arming wears off on its own.
    pub fn stop_clicked(&mut self, id: Uuid, cx: &mut Context<Self>) {
        let s = self.sync.entry(id).or_default();
        if s.stop_armed {
            s.stop_armed = false;
            self.kill(id);
            return;
        }
        s.stop_armed = true;
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            executor.timer(std::time::Duration::from_secs(4)).await;
            let _ = this.update(cx, |ui, cx| {
                if let Some(s) = ui.sync.get_mut(&id) {
                    s.stop_armed = false;
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn kill(&mut self, id: Uuid) {
        self.backend
            .send(MessageToBackend::KillGame { server_id: id });
    }

    /// Enabling is checked against the build's rules: a conflicting mod, or one
    /// missing a dependency, stays off and the player is told why. The other
    /// side of a conflict is never switched off for them.
    pub fn toggle_optional(&mut self, server_id: Uuid, name: &str, cx: &mut Context<Self>) {
        if let Some(mods) = self.optional_mods.get_mut(&server_id) {
            let turning_on = mods
                .iter()
                .find(|m| m.name == name)
                .is_some_and(|m| !m.enabled);
            if turning_on {
                if let Some((text, level)) = Self::blocking_issue(mods, name) {
                    self.notify_toast(text, level, cx);
                    return;
                }
            }
            if let Some(m) = mods.iter_mut().find(|m| m.name == name) {
                if !m.allowed {
                    return;
                }
                m.enabled = !m.enabled;
            }
            let enabled: Vec<String> = mods
                .iter()
                .filter(|m| m.enabled)
                .map(|m| m.name.clone())
                .collect();
            self.backend
                .send(MessageToBackend::SetOptionalMods { server_id, enabled });
        }
    }

    /// What stands in the way of enabling the mod. `None` means it can go on.
    ///
    /// The rules are shared with the master (`schema::optional`); let them drift
    /// apart and the launcher would allow what the master then rejects.
    fn blocking_issue(mods: &[OptionalModInfo], name: &str) -> Option<(String, NotifLevel)> {
        let known: Vec<schema::build::OptionalMod> = mods.iter().map(Self::as_rule).collect();
        let enabled: Vec<String> = mods
            .iter()
            .filter(|m| m.enabled)
            .map(|m| m.name.clone())
            .collect();
        let issue = schema::optional::can_enable(&known, &enabled, name).err()?;
        let mut args = i18n::FluentArgs::new();
        let key = match &issue {
            schema::optional::SelectionIssue::Conflict { with, .. } => {
                args.set("mod", with.clone());
                "optional-conflicts-with"
            }
            schema::optional::SelectionIssue::MissingDependency { needs, .. } => {
                args.set("mod", needs.clone());
                "optional-needs-first"
            }
        };
        Some((i18n::t_args(key, &args), NotifLevel::Warning))
    }

    /// Only the name and the links matter to `can_enable`, so the rest of the
    /// rule is filled in with blanks.
    fn as_rule(m: &OptionalModInfo) -> schema::build::OptionalMod {
        schema::build::OptionalMod {
            name: m.name.clone(),
            description: String::new(),
            category: String::new(),
            files: Vec::new(),
            enabled_by_default: false,
            visible: true,
            limited: m.limited,
            dependencies: m.dependencies.clone(),
            conflicts: m.conflicts.clone(),
            triggers: Vec::new(),
            os: Vec::new(),
            icon_url: None,
            author: None,
        }
    }

    /// `None` goes back to the currently published build. The backend keeps the
    /// choice and re-requests the manifest, so the file and mod lists follow on
    /// their own.
    pub fn select_build(&mut self, server_id: Uuid, build_id: Option<Uuid>) {
        self.selected_build.insert(server_id, build_id);
        self.backend.send(MessageToBackend::SelectBuild {
            server_id,
            build_id,
        });
    }

    pub fn set_memory(&mut self, min_mb: u32, max_mb: u32) {
        self.config.memory_min_mb = min_mb;
        self.config.memory_max_mb = max_mb;
        self.backend
            .send(MessageToBackend::SetMemory { min_mb, max_mb });
    }

    pub fn set_server_memory(&mut self, server_id: Uuid, min_mb: u32, max_mb: u32) {
        let mut settings = self.server_client_settings(server_id);
        settings.memory_min_mb = min_mb;
        settings.memory_max_mb = max_mb;
        self.server_settings.insert(server_id, settings);
        self.backend.send(MessageToBackend::SetServerMemory {
            server_id,
            min_mb,
            max_mb,
        });
    }

    pub fn set_show_console_on_launch(&mut self, enabled: bool) {
        self.config.show_console_on_launch = enabled;
        self.backend
            .send(MessageToBackend::SetShowConsoleOnLaunch { enabled });
    }

    pub fn set_fullscreen(&mut self, enabled: bool) {
        self.config.fullscreen = enabled;
        self.backend
            .send(MessageToBackend::SetFullscreen { enabled });
    }

    /// Takes effect on the next launch: Sentry comes up before GPUI, and its
    /// panic hook can't be removed while the process runs.
    pub fn set_discord_rpc(&mut self, enabled: bool) {
        self.config.discord_rpc = enabled;
        self.backend
            .send(MessageToBackend::SetDiscordRpc { enabled });
    }

    pub fn set_crash_reports(&mut self, enabled: bool) {
        self.config.crash_reports = enabled;
        self.backend
            .send(MessageToBackend::SetCrashReports { enabled });
    }

    pub fn answer_log_request(&mut self, accepted: bool) {
        let Some(prompt) = self.log_request_prompt.take() else {
            return;
        };
        self.backend.send(MessageToBackend::LogRequestAnswer {
            request_id: prompt.request_id,
            accepted,
        });
    }

    pub fn answer_remote_action(&mut self, accepted: bool) {
        let Some(prompt) = self.remote_action_prompt.take() else {
            return;
        };
        self.backend.send(MessageToBackend::RemoteActionAnswer {
            action: prompt.action,
            server_id: prompt.server_id,
            accepted,
        });
    }

    /// Close the forced-collection modal; there is nothing to answer there.
    pub fn dismiss_log_request(&mut self) {
        self.log_request_prompt = None;
    }

    pub fn answer_impersonate(&mut self, accepted: bool) {
        let Some(prompt) = self.impersonate_prompt.take() else {
            return;
        };
        self.backend.send(MessageToBackend::ImpersonateAnswer {
            grant_id: prompt.grant_id,
            accepted,
        });
    }

    pub fn exit_impersonation(&mut self) {
        self.backend.send(MessageToBackend::ImpersonateExit);
    }

    /// No server id: the backend picks the one whose manifest is already loaded.
    pub fn send_support_bundle(&mut self) {
        self.backend
            .send(MessageToBackend::SendSupportBundle { server_id: None });
    }

    pub fn set_server_show_console_on_launch(&mut self, server_id: Uuid, enabled: bool) {
        let mut settings = self.server_client_settings(server_id);
        settings.show_console_on_launch = enabled;
        self.server_settings.insert(server_id, settings);
        self.backend
            .send(MessageToBackend::SetServerShowConsoleOnLaunch { server_id, enabled });
    }

    pub fn set_server_fullscreen(&mut self, server_id: Uuid, enabled: bool) {
        let mut settings = self.server_client_settings(server_id);
        settings.fullscreen = enabled;
        self.server_settings.insert(server_id, settings);
        self.backend
            .send(MessageToBackend::SetServerFullscreen { server_id, enabled });
    }

    pub fn set_server_jvm_flags(&mut self, server_id: Uuid, flags: String) {
        let mut settings = self.server_client_settings(server_id);
        settings.jvm_flags = flags.clone();
        self.server_settings.insert(server_id, settings);
        self.backend
            .send(MessageToBackend::SetServerJvmFlags { server_id, flags });
    }

    pub fn reset_server_client_settings(&mut self, server_id: Uuid) {
        self.server_settings.remove(&server_id);
        self.backend
            .send(MessageToBackend::ResetServerClientSettings { server_id });
    }

    pub fn open_server_client_folder(&mut self, server_id: Uuid) {
        self.backend
            .send(MessageToBackend::OpenServerClientFolder { server_id });
    }

    /// Switch the UI language. The backend pulls the catalog from the master.
    pub fn set_locale(&mut self, locale: i18n::Locale) {
        if self.locale == locale {
            return;
        }
        self.locale = locale;
        i18n::set_locale(locale);
        self.backend.send(MessageToBackend::SetLocale {
            code: locale.code().to_string(),
        });
    }

    pub fn install_update(&mut self, cx: &mut Context<Self>) {
        if self.updating {
            return;
        }
        let Some(v) = self.update_available.clone() else {
            return;
        };
        self.updating = true;
        let modal = bridge::ModalAction::new("Update");
        self.update_modal = Some(modal.clone());
        self.backend.send(MessageToBackend::InstallUpdate {
            version: v,
            modal_action: modal.clone(),
        });
        // The download reports into the modal, not through messages: redraw a
        // few times a second while it runs, and stand down if it fails. On
        // success the launcher restarts into the new version.
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| loop {
            executor.timer(std::time::Duration::from_millis(250)).await;
            let progress = modal.snapshot();
            let done = progress.error.is_some() || progress.finished;
            let alive = this.update(cx, |ui, cx| {
                if progress.error.is_some() {
                    ui.updating = false;
                    ui.update_modal = None;
                }
                cx.notify();
            });
            if done || alive.is_err() {
                break;
            }
        })
        .detach();
    }

    pub fn toggle_console(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_server_id() {
            self.open_console(id, cx);
        }
    }

    pub fn open_console(&mut self, server_id: Uuid, cx: &mut Context<Self>) {
        if let Some(handle) = &self.console_window {
            let lines: Vec<LogEntry> = self
                .logs
                .get(&server_id)
                .map(|l| l.iter().cloned().collect())
                .unwrap_or_default();
            let _ = handle.update(cx, |view, window, cx| {
                if view.server_id != server_id {
                    view.show_server(server_id, lines);
                }
                window.activate_window();
                cx.notify();
            });
            return;
        }

        let bounds = gpui::Bounds::centered(
            None,
            gpui::size(px(CONSOLE_WINDOW_SIZE.0), px(CONSOLE_WINDOW_SIZE.1)),
            cx,
        );
        let logs: Vec<LogEntry> = self
            .logs
            .get(&server_id)
            .map(|l| l.iter().cloned().collect())
            .unwrap_or_default();
        let handle = cx.open_window(
            gpui::WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                window_min_size: Some(gpui::size(
                    px(CONSOLE_WINDOW_MIN_SIZE.0),
                    px(CONSOLE_WINDOW_MIN_SIZE.1),
                )),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some(i18n::t("console-title").into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |_, cx| {
                cx.new(|cx| {
                    cx.on_release(|_: &mut ConsoleWindow, cx| {
                        if let Some(ui) = cx.try_global::<GlobalLauncherUI>() {
                            let ui = ui.0.clone();
                            ui.update(cx, |this_ui, cx| {
                                this_ui.console_window = None;
                                cx.notify();
                            });
                        }
                    })
                    .detach();
                    ConsoleWindow::new(server_id, logs)
                })
            },
        );

        match handle {
            Ok(h) => self.console_window = Some(h),
            Err(e) => tracing::warn!(error = %e, "console window did not open"),
        }
    }
}
