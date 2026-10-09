// Over 150 lines: the window's small types; each is short, and they are read
// together.
//! The window's smaller types: pages, sync state, toasts, prompts.

use super::*;

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
    /// The kill went out and the process hasn't exited yet. Without it the
    /// button went on offering «Stop» and armed itself again on a click.
    pub stopping: bool,
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

pub(super) fn translate_notification(
    key: &str,
    args: &std::collections::BTreeMap<String, String>,
) -> String {
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
    pub console: bridge::ConsoleSettings,
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
            console: bridge::ConsoleSettings::default(),
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

/// Preset and cape cards are 72 px wide. The master renders at 8 to 10 times
/// the texture size; twice the card is plenty on a HiDPI screen.
pub(super) const PRESET_RENDER_SIDE: u32 = 192;

#[derive(Clone, Debug)]
pub struct SavedSkinPreset {
    pub id: String,
    pub name: String,
    /// Shared: the preset cards are drawn every frame the skin turns, and
    /// copying each preset's PNG for every one of them added up.
    pub bytes: Arc<Vec<u8>>,
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
    pub expires_at: std::time::Instant,
}

impl ImpersonatePrompt {
    pub fn seconds_left(&self) -> i64 {
        self.expires_at
            .saturating_duration_since(std::time::Instant::now())
            .as_secs() as i64
    }
}
