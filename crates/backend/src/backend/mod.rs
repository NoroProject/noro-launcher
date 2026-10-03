//! Backend core: state, the main event loop, launching the game.

use crate::auth::token_store;
use crate::config::{LauncherConfig, OptionalModsSelection};
use crate::directories::LauncherDirectories;
use crate::game_runner::{self, LoginInfo, ServerConnect};
use crate::persistent::Persistent;
use crate::ws_client::{self, WsClient};
use bridge::{BackendReceiver, FrontendHandle, MessageToBackend, MessageToFrontend, QuitHandler};
use parking_lot::Mutex;
use schema::{BuildManifest, ClientWsMsg, ServerEntry, ServerWsMsg, UserProfile};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

pub struct RunningGame {
    pub started: Instant,
    /// Send here to kill the process.
    pub kill: UnboundedSender<()>,
}

/// A background task asking the main loop to change state — the loop owns it,
/// the tasks only have a `Ctx`.
pub enum InternalEvent {
    LoginCompleted {
        auth: token_store::StoredAuth,
        user: UserProfile,
    },
    LoginFailed {
        kind: bridge::LoginErrorKind,
    },
    /// Update installed; restart from the new binary.
    RestartInto(std::path::PathBuf),
    /// Skin/cape/profile change from native upload or external — refresh UI.
    ProfileUpdated {
        user: UserProfile,
    },
    /// The grant was traded in; switch the session to that player's account.
    ImpersonationStarted {
        access_token: String,
        username: String,
    },
    /// The stored session checked out at startup.
    SessionRestored {
        user: UserProfile,
    },
    /// A refresh traded the old tokens for new ones.
    TokensRefreshed {
        auth: token_store::StoredAuth,
    },
    /// The master turned the session down and the refresh token with it.
    SessionRejected,
    /// The master couldn't be reached to say either way. The session is kept:
    /// unreachable is not the same as rejected.
    SessionUnverified,
    /// A launch has waited too long for its manifest.
    ManifestTimeout {
        server_id: Uuid,
        seq: u64,
    },
    /// Jar icons for a build's optional mods have been read in the background.
    JarIconsReady {
        server_id: Uuid,
    },
}

/// What a background task gets: everything shared, nothing owned by the loop.
#[derive(Clone)]
pub struct Ctx {
    pub frontend: FrontendHandle,
    pub ws: WsClient,
    pub http: reqwest::Client,
    pub dirs: LauncherDirectories,
    pub config: Persistent<LauncherConfig>,
    pub optional: Persistent<OptionalModsSelection>,
    pub running: Arc<Mutex<HashMap<Uuid, RunningGame>>>,
    pub internal: UnboundedSender<InternalEvent>,
    pub rpc: crate::discord_rpc::DiscordRpc,
    /// Channel to the in-game case mod. Lives as long as the launcher, but the
    /// listener is only up while a game is running.
    pub mod_link: crate::mod_link::ModLink,
    /// A copy of what the main loop holds. The case panel needs the profile
    /// from a background task, and dragging all of `BackendState` there isn't
    /// worth it.
    pub(crate) profile: Arc<parking_lot::RwLock<Option<UserProfile>>>,
}

impl Ctx {
    pub fn send(&self, msg: MessageToFrontend) {
        self.frontend.send(msg);
    }

    pub fn profile(&self) -> Option<UserProfile> {
        self.profile.read().clone()
    }

    pub fn set_profile(&self, user: Option<UserProfile>) {
        *self.profile.write() = user;
    }
}

/// Owned by the main loop and never shared.
pub struct BackendState {
    pub ctx: Ctx,
    pub rx_backend: BackendReceiver,
    pub quit: QuitHandler,
    pub master_rx: UnboundedReceiver<ServerWsMsg>,
    pub conn_rx: UnboundedReceiver<bool>,
    pub internal_rx: UnboundedReceiver<InternalEvent>,

    pub user: Option<UserProfile>,
    pub access_token: Option<String>,
    /// Our own token, parked here while impersonating someone. Leaving their
    /// account restores this rather than asking for a fresh login.
    pub own_token: Option<String>,
    pub servers: Vec<ServerEntry>,
    pub manifests: HashMap<Uuid, BuildManifest>,
    /// Installed builds' manifests from the disk. Only for when the master
    /// doesn't answer: online, a launch always asks for the current one.
    pub cached_manifests: HashMap<Uuid, BuildManifest>,
    /// Whether the socket to the master is up.
    pub online: bool,
    /// The build whose file list the window already has, per server.
    pub files_sent_for: HashMap<Uuid, Uuid>,
    /// Launches waiting on a manifest to arrive.
    pub pending_launch: HashMap<Uuid, bridge::ModalAction>,
    /// Builds whose state the window already has. A guess from the disk is
    /// only for the rest: it must never replace what a manifest established.
    pub build_state_known: HashSet<Uuid>,
    /// The server the player last pressed Play for. Logs for "report a
    /// problem" come from there: the game writes them into that instance.
    pub last_launched: Option<Uuid>,
    /// Bumped per launch request, so a stale timeout can't fail a newer launch
    /// of the same server.
    pub launch_seq: HashMap<Uuid, u64>,
    /// A token refresh is on its way; further auth failures wait for it.
    pub refresh_in_flight: bool,
    /// When the last refresh succeeded. Failing again right after one means
    /// the new token is no good either, and refreshing again would loop.
    pub last_refresh: Option<Instant>,
}

mod launch;
mod session;
mod state;

pub use launch::{spawn_sync_and_launch, Launch};
use session::{check_launcher_update, restore_session};
pub use session::{refresh_tokens, RefreshError};

/// Spawns the main loop onto `runtime` and returns immediately.
pub fn start(
    runtime: &tokio::runtime::Runtime,
    tx_frontend: FrontendHandle,
    rx_backend: BackendReceiver,
    quit: QuitHandler,
) {
    runtime.spawn(async move {
        if let Err(e) = run(tx_frontend, rx_backend, quit).await {
            tracing::error!("backend stopped with an error: {e:#}");
        }
    });
}

async fn run(
    tx_frontend: FrontendHandle,
    rx_backend: BackendReceiver,
    quit: QuitHandler,
) -> anyhow::Result<()> {
    let dirs = LauncherDirectories::new();
    dirs.ensure().ok();

    let config = Persistent::<LauncherConfig>::load(dirs.config_file());
    config.update(|c| {
        if let Some(old) = c.adopt_stamped_master() {
            tracing::info!(
                "master address from the bootstrapper: {old} -> {}",
                c.master_url
            );
        }
        if c.fix_localhost() {
            tracing::info!("migrated config: localhost -> 127.0.0.1");
        }
    });
    let optional = Persistent::<OptionalModsSelection>::load(dirs.optional_mods_file());
    let http = crate::http::client()?;

    let stored = token_store::load();
    if stored.is_some() {
        tracing::info!("session loaded from the keyring");
    } else {
        tracing::info!("no session in the keyring");
    }
    let access_token = stored.as_ref().map(|s| s.access_token.clone());

    let (master_tx, master_rx) = mpsc::unbounded_channel::<ServerWsMsg>();
    let (conn_tx, conn_rx) = mpsc::unbounded_channel::<bool>();
    let (internal_tx, internal_rx) = mpsc::unbounded_channel::<InternalEvent>();
    let ws = ws_client::spawn(
        config.get().ws_url(),
        access_token.clone(),
        master_tx,
        conn_tx,
    );

    tracing::info!("using master server: {}", config.get().master_url);

    let rpc = crate::discord_rpc::spawn_discord_rpc(config.get().discord_rpc);
    rpc.update(crate::discord_rpc::DiscordRpcState::Launcher { server_name: None });

    let ctx = Ctx {
        frontend: tx_frontend,
        ws,
        http,
        dirs,
        config,
        optional,
        running: Arc::new(Mutex::new(HashMap::new())),
        internal: internal_tx,
        rpc,
        mod_link: crate::mod_link::ModLink::default(),
        profile: Arc::new(parking_lot::RwLock::new(None)),
    };

    let mut state = BackendState {
        ctx,
        rx_backend,
        quit,
        master_rx,
        conn_rx,
        internal_rx,
        user: None,
        access_token,
        own_token: None,
        servers: Vec::new(),
        manifests: HashMap::new(),
        cached_manifests: HashMap::new(),
        online: false,
        files_sent_for: HashMap::new(),
        pending_launch: HashMap::new(),
        build_state_known: HashSet::new(),
        last_launched: None,
        launch_seq: HashMap::new(),
        refresh_in_flight: false,
        last_refresh: None,
    };

    // Settings and the cached catalog first: they need no network, and the
    // window should be in the player's language from the first frame rather
    // than after the master answers.
    state.send_config_state();
    crate::translations::refresh(&state.ctx, state.ctx.config.get().locale);
    state.load_offline_cache();

    // Over REST rather than waiting for the socket: the login screen would
    // otherwise flash by on every start. In the background, so the loop below
    // takes the window's requests meanwhile.
    match state.access_token.clone() {
        Some(token) => {
            tokio::spawn(restore_session(state.ctx.clone(), token));
        }
        None => state.ctx.send(MessageToFrontend::SessionCheckDone),
    }
    tokio::spawn(check_launcher_update(state.ctx.clone()));

    state.main_loop().await;
    Ok(())
}
