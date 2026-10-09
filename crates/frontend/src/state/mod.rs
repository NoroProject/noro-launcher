// Over 150 lines: the window state struct; its fields are listed once, here.
//! UI state, and the handling of messages from the backend.

use bridge::{
    BackendHandle, ClientSettingsState, LoginErrorKind, MessageToBackend, MessageToFrontend,
    OptionalModInfo, SyncStage,
};
use gpui::{px, AppContext, Context, Entity, Image, RenderImage};

use schema::{LauncherVersion, NewsItem, NotifLevel, ServerEntry, UserProfile};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

mod console;
mod images;
mod init;
mod messages;
mod messages_account;
mod messages_content;
mod messages_session;
mod messages_sync;
mod servers;
mod session;
mod settings;
mod types;

pub use console::ConsoleWindow;
pub use types::*;

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
    /// Normalised names of a build's files. Kept apart from the optional mod
    /// names in `installed_keys`, because the file list only comes when the
    /// build changes.
    installed_file_keys: HashMap<Uuid, std::collections::HashSet<String>>,
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
    /// Which of them have answered. Asked but not answered is "loading", not
    /// "nothing here": a player with punishments used to see "none" for the
    /// second it took the master to reply.
    pub account_loaded: HashSet<&'static str>,
    pub dm_loaded: bool,
    pub news_loaded: bool,
    /// Window placement and the last open server, written on quit.
    pub ui_state: crate::ui_state::UiState,
    /// The saved server has been reopened once; after that the player's own
    /// clicks decide.
    last_server_restored: bool,
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
    /// Conversations open on their newest message, like any chat; without a
    /// handle they opened at the top, on the oldest one.
    pub dm_scroll: gpui::ScrollHandle,
    pub ticket_scroll: gpui::ScrollHandle,
    /// What is being typed, in whichever of the two is open.
    pub compose: String,
    pub compose_focus: Option<gpui::FocusHandle>,
}

pub struct GlobalLauncherUI(pub Entity<LauncherUI>);
impl gpui::Global for GlobalLauncherUI {}

use crate::console_window::MAX_LOG_LINES;

/// Catalogue icons and avatars are drawn at a few dozen pixels.
const ICON_SIDE: u32 = 128;
/// Mod screenshots fill a wide gallery box.
const SCREENSHOT_SIDE: u32 = 480;
/// Remote pictures kept decoded at once before the cache starts over.
const REMOTE_IMAGE_CAP: usize = 256;

fn trimmed(url: Option<&String>) -> Option<&str> {
    url.map(|u| u.trim()).filter(|u| !u.is_empty())
}
