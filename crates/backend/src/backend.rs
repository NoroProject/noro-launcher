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

const STARTUP_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// The update check is a nicety; it must not hold anything up.
const UPDATE_CHECK_TIMEOUT: Duration = Duration::from_secs(15);

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

    let rpc = crate::discord_rpc::spawn_discord_rpc();
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

impl BackendState {
    async fn main_loop(&mut self) {
        loop {
            tokio::select! {
                Some(msg) = self.rx_backend.recv() => {
                    let quit = matches!(msg, MessageToBackend::Quit);
                    self.handle_to_backend(msg).await;
                    if quit {
                        break;
                    }
                }
                Some(msg) = self.master_rx.recv() => {
                    self.handle_from_master(msg).await;
                }
                Some(online) = self.conn_rx.recv() => {
                    self.ctx.send(MessageToFrontend::ConnectionState { online });
                    if online {
                        // Both lists may have moved on while we were offline.
                        self.ctx.ws.send(ClientWsMsg::RequestServerList);
                        self.ctx.ws.send(ClientWsMsg::RequestNews);
                    }
                }
                Some(event) = self.internal_rx.recv() => {
                    self.handle_internal(event);
                }
                else => break,
            }
        }
        tracing::info!("backend: main loop finished");
        // Checking in with the coordinator happens when `self.quit` is
        // dropped together with the state — on an error or a panic as well.
    }

    fn handle_internal(&mut self, event: InternalEvent) {
        match event {
            InternalEvent::LoginCompleted { auth, user } => {
                if let Err(e) = token_store::save(&auth) {
                    tracing::error!("could not save the session to the keyring: {e}");
                } else {
                    tracing::info!("session saved to the keyring");
                }
                self.access_token = Some(auth.access_token.clone());
                self.user = Some(user.clone());
                self.ctx.ws.set_token(Some(auth.access_token));
                self.ctx.send(MessageToFrontend::LoginSuccess { user });
                self.ctx.send(MessageToFrontend::CloseModal);
            }
            InternalEvent::LoginFailed { kind } => {
                self.ctx.send(MessageToFrontend::LoginFailed { kind });
                self.ctx.send(MessageToFrontend::CloseModal);
            }
            InternalEvent::RestartInto(exe) => {
                // Returns only if the new binary didn't start; the old one
                // keeps running rather than leaving nothing.
                crate::updater::restart(&exe);
                self.ctx.send(MessageToFrontend::AddNotification {
                    key: "notif-update-failed".into(),
                    args: [(
                        "reason".to_string(),
                        format!("could not start {}", exe.display()),
                    )]
                    .into(),
                    level: schema::NotifLevel::Error,
                });
            }
            InternalEvent::ProfileUpdated { user } => {
                self.user = Some(user.clone());
                self.ctx
                    .send(MessageToFrontend::PermissionsUpdated { user });
            }
            InternalEvent::ImpersonationStarted {
                access_token,
                username,
            } => {
                // Deliberately not saved to the keyring: someone else's session
                // lasts half an hour and must not survive a restart.
                self.access_token = Some(access_token.clone());
                self.ctx.ws.set_token(Some(access_token));
                self.ctx.send(MessageToFrontend::ImpersonationChanged {
                    as_username: Some(username),
                });
            }
            InternalEvent::SessionRestored { user } => {
                self.user = Some(user.clone());
                self.ctx.set_profile(Some(user.clone()));
                self.ctx.send(MessageToFrontend::LoginSuccess { user });
            }
            InternalEvent::TokensRefreshed { auth } => {
                self.refresh_in_flight = false;
                self.last_refresh = Some(Instant::now());
                if let Err(e) = token_store::save(&auth) {
                    // Rotated refresh tokens make the old one useless, so the
                    // next start will ask for a login; say why in the log.
                    tracing::error!("refreshed session not saved to the keyring: {e:#}");
                }
                self.access_token = Some(auth.access_token.clone());
                self.ctx.ws.set_token(Some(auth.access_token));
            }
            InternalEvent::SessionRejected => {
                self.refresh_in_flight = false;
                self.sign_out();
            }
            InternalEvent::SessionUnverified => {
                self.refresh_in_flight = false;
                if self.user.is_none() {
                    self.ctx.send(MessageToFrontend::SessionCheckDone);
                }
            }
            InternalEvent::ManifestTimeout { server_id, seq } => {
                if self.launch_seq.get(&server_id) != Some(&seq) {
                    return;
                }
                if let Some(modal) = self.pending_launch.remove(&server_id) {
                    modal.fail("notif-manifest-timeout");
                    self.ctx
                        .send(MessageToFrontend::LaunchCancelled { server_id });
                    self.ctx.send(MessageToFrontend::AddNotification {
                        key: "notif-manifest-timeout".into(),
                        args: BTreeMap::new(),
                        level: schema::NotifLevel::Error,
                    });
                }
            }
        }
    }

    /// Forget the session everywhere: keyring, socket, window. A borrowed
    /// session goes with it, and so does the banner that announced it.
    pub fn sign_out(&mut self) {
        if let Err(e) = token_store::clear() {
            tracing::error!("could not remove the stored session: {e:#}");
        }
        if self.own_token.take().is_some() {
            self.ctx
                .send(MessageToFrontend::ImpersonationChanged { as_username: None });
        }
        self.access_token = None;
        self.user = None;
        self.ctx.set_profile(None);
        self.ctx.ws.set_token(None);
        self.ctx.send(MessageToFrontend::LoggedOut);
    }

    /// The socket refused our token. A borrowed session simply ran out; ours
    /// gets one refresh before the player is signed out.
    pub fn on_auth_failed(&mut self) {
        if self.own_token.is_some() {
            self.exit_impersonation();
            self.ctx.send(MessageToFrontend::AddNotification {
                key: "notif-impersonate-ended".into(),
                args: BTreeMap::new(),
                level: schema::NotifLevel::Warning,
            });
            return;
        }
        if self.refresh_in_flight {
            return;
        }
        if self
            .last_refresh
            .is_some_and(|at| at.elapsed() < Duration::from_secs(30))
        {
            self.sign_out();
            return;
        }
        self.refresh_in_flight = true;
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let event = match refresh_tokens(&ctx).await {
                Ok(auth) => InternalEvent::TokensRefreshed { auth },
                Err(RefreshError::Unreachable) => InternalEvent::SessionUnverified,
                Err(RefreshError::Rejected) => InternalEvent::SessionRejected,
            };
            let _ = ctx.internal.send(event);
        });
    }

    /// `None` until both the profile and the token are in hand — the game can't
    /// be started with half a session.
    pub fn login_info(&self) -> Option<LoginInfo> {
        let user = self.user.as_ref()?;
        let token = self.access_token.clone()?;
        Some(LoginInfo {
            username: user.username.clone(),
            uuid: user.uuid.simple().to_string(),
            access_token: token,
        })
    }

    /// No address means no auto-connect — the game just opens on the main menu.
    pub fn server_connect(&self, server_id: &Uuid) -> Option<ServerConnect> {
        self.servers
            .iter()
            .find(|s| &s.id == server_id)
            .and_then(|s| Some((s.mc_host.clone()?, s.mc_port?)))
            .map(|(host, port)| ServerConnect { host, port })
    }
}

/// Why a refresh didn't produce new tokens.
pub enum RefreshError {
    /// No refresh token, or the master refused it: the session is over.
    Rejected,
    /// The master couldn't be reached; try again later.
    Unreachable,
}

fn master_base(ctx: &Ctx) -> String {
    ctx.config
        .get()
        .master_url
        .trim_end_matches('/')
        .to_string()
}

/// Trade the stored refresh token for a new pair.
pub async fn refresh_tokens(ctx: &Ctx) -> Result<token_store::StoredAuth, RefreshError> {
    let Some(stored) = token_store::load().filter(|s| !s.refresh_token.is_empty()) else {
        tracing::info!("no refresh token in the keyring");
        return Err(RefreshError::Rejected);
    };
    let resp = ctx
        .http
        .post(format!("{}/auth/refresh", master_base(ctx)))
        .json(&serde_json::json!({ "refresh_token": stored.refresh_token }))
        .timeout(STARTUP_REQUEST_TIMEOUT)
        .send()
        .await;
    let resp = match resp {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("token refresh: master unreachable: {e}");
            return Err(RefreshError::Unreachable);
        }
    };
    if resp.status().is_server_error() {
        tracing::warn!("token refresh: master returned {}", resp.status());
        return Err(RefreshError::Unreachable);
    }
    if !resp.status().is_success() {
        tracing::info!("token refresh refused: {}", resp.status());
        return Err(RefreshError::Rejected);
    }
    let v: serde_json::Value = resp.json().await.map_err(|_| RefreshError::Unreachable)?;
    match (v["access_token"].as_str(), v["refresh_token"].as_str()) {
        (Some(at), Some(rt)) => {
            tracing::info!("token refreshed");
            Ok(token_store::StoredAuth {
                access_token: at.to_string(),
                refresh_token: rt.to_string(),
            })
        }
        _ => {
            tracing::warn!("token refresh: response had no tokens in it");
            Err(RefreshError::Unreachable)
        }
    }
}

enum MeError {
    Rejected,
    Unreachable,
}

async fn fetch_me(ctx: &Ctx, token: &str) -> Result<UserProfile, MeError> {
    let resp = ctx
        .http
        .get(format!("{}/api/me", master_base(ctx)))
        .bearer_auth(token)
        .timeout(STARTUP_REQUEST_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!("restore_session: master unreachable: {e}");
            MeError::Unreachable
        })?;
    match resp.status().as_u16() {
        401 | 403 => Err(MeError::Rejected),
        s if !(200..300).contains(&s) => {
            tracing::warn!("restore_session: master returned {s}");
            Err(MeError::Unreachable)
        }
        _ => resp.json::<UserProfile>().await.map_err(|e| {
            tracing::warn!("restore_session: profile did not parse: {e}");
            MeError::Unreachable
        }),
    }
}

/// Check the stored session at startup, refreshing it once if it expired.
/// Reports back through `InternalEvent`s; never logs the player out over a
/// network problem.
async fn restore_session(ctx: Ctx, token: String) {
    let event = match fetch_me(&ctx, &token).await {
        Ok(user) => {
            // No name here: this log goes out with "report a problem".
            tracing::info!("restore_session: session restored");
            InternalEvent::SessionRestored { user }
        }
        Err(MeError::Unreachable) => InternalEvent::SessionUnverified,
        Err(MeError::Rejected) => match refresh_tokens(&ctx).await {
            Ok(auth) => {
                let access = auth.access_token.clone();
                let _ = ctx.internal.send(InternalEvent::TokensRefreshed { auth });
                match fetch_me(&ctx, &access).await {
                    Ok(user) => InternalEvent::SessionRestored { user },
                    Err(MeError::Rejected) => InternalEvent::SessionRejected,
                    Err(MeError::Unreachable) => InternalEvent::SessionUnverified,
                }
            }
            Err(RefreshError::Rejected) => InternalEvent::SessionRejected,
            Err(RefreshError::Unreachable) => InternalEvent::SessionUnverified,
        },
    };
    let _ = ctx.internal.send(event);
}

/// Background check for a newer launcher. Silent on any failure: being offline
/// is no reason to bother the player, and the bootstrapper checks too.
async fn check_launcher_update(ctx: Ctx) {
    let url = format!(
        "{}/api/launcher/version?platform={}",
        master_base(&ctx),
        schema::current_platform()
    );
    let Ok(Ok(r)) = tokio::time::timeout(UPDATE_CHECK_TIMEOUT, ctx.http.get(&url).send()).await
    else {
        return;
    };
    let Ok(v) = r.json::<serde_json::Value>().await else {
        return;
    };
    let Some(version) = v["version"].as_str() else {
        return;
    };
    // The master reports a git tag, "v1.2.0" (or the legacy "launcher-v1.2.0"),
    // while the crate exposes "1.2.0". Without stripping the prefix they never
    // match and the update banner is always up.
    let reported = version
        .trim_start_matches("launcher-")
        .trim_start_matches('v');
    if reported == env!("CARGO_PKG_VERSION") {
        return;
    }
    if let Ok(lv) = serde_json::from_value::<schema::LauncherVersion>(build_launcher_version(&v)) {
        ctx.send(MessageToFrontend::LauncherUpdateAvailable { version: lv });
    }
}

/// `/api/launcher/version` answers with a subset of `LauncherVersion`; the rest
/// is filled in here so it can be deserialized as one.
fn build_launcher_version(v: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "id": Uuid::nil(),
        "version": v["version"],
        "platform": v["platform"],
        "url": v["url"],
        "sha256": v["sha256"],
        "signature": v["signature"],
        "is_current": true,
    })
}

const PROGRESS_INTERVAL: Duration = Duration::from_millis(50);

/// Everything needed to sync a build and start the game.
pub struct Launch {
    pub ctx: Ctx,
    pub server_id: Uuid,
    pub manifest: BuildManifest,
    pub user: UserProfile,
    pub login: LoginInfo,
    pub connect: Option<ServerConnect>,
    pub enabled_optional: Vec<String>,
    /// The build's card; its game servers end up in the instance's servers.dat.
    pub server: Option<ServerEntry>,
    pub modal: bridge::ModalAction,
}

pub fn spawn_sync_and_launch(req: Launch) {
    let Launch {
        ctx,
        server_id,
        manifest,
        user,
        login,
        connect,
        enabled_optional,
        server,
        modal,
    } = req;
    tokio::spawn(async move {
        let instance_dir = ctx.dirs.instance(&server_id);

        let to_fe = ctx.frontend.clone();
        let modal_clone = modal.clone();
        // Download stages run in parallel but the modal has one bar. Keeping the
        // last report per stage and showing the sum stops the bar jumping back
        // and forth with whichever stage reported last.
        let totals: Arc<Mutex<BTreeMap<bridge::SyncStage, (u64, u64)>>> =
            Arc::new(Mutex::new(BTreeMap::new()));
        // The window hears from each stage every 50ms at most, plus its first
        // and last report. A sync reports every file, and the 13 000 messages
        // of one GTNH install buried the one saying the sync was over.
        let last_sent: Arc<Mutex<BTreeMap<bridge::SyncStage, Instant>>> =
            Arc::new(Mutex::new(BTreeMap::new()));
        let progress: crate::sync::ProgressFn = Arc::new(move |stage, done, total, file| {
            let due = {
                let mut last = last_sent.lock();
                let now = Instant::now();
                let due = done == 0
                    || done >= total
                    || last
                        .get(&stage)
                        .is_none_or(|at| now.duration_since(*at) >= PROGRESS_INTERVAL);
                if due {
                    last.insert(stage, now);
                }
                due
            };
            if due {
                to_fe.send(MessageToFrontend::SyncProgress {
                    server_id,
                    stage,
                    done,
                    total,
                    file: file.clone(),
                });
            }
            if stage.is_download() {
                let (sum_done, sum_total) = {
                    let mut g = totals.lock();
                    g.insert(stage, (done, total));
                    g.values()
                        .fold((0u64, 0u64), |(d, t), (sd, st)| (d + sd, t + st))
                };
                modal_clone.set_stage("downloading");
                modal_clone.set_progress(sum_done, sum_total);
            } else {
                modal_clone.set_stage(format!("{stage:?}"));
                modal_clone.set_progress(done, total);
            }
            if !file.is_empty() {
                modal_clone.set_detail(file);
            }
        });

        let cancelled_modal = modal.clone();
        let cancelled: Arc<dyn Fn() -> bool + Send + Sync> =
            Arc::new(move || cancelled_modal.is_cancelled());

        let sync_result = crate::sync::sync_server(
            &ctx.http,
            &instance_dir,
            &manifest,
            &enabled_optional,
            &user,
            progress,
            cancelled,
        )
        .await;

        if let Err(e) = sync_result {
            // A cancel surfaces as an error from deep inside the download; it
            // is the player's choice, not a failure to report.
            if modal.is_cancelled() {
                ctx.send(MessageToFrontend::LaunchCancelled { server_id });
                return;
            }
            // `{:#}` keeps the cause chain: "download of X failed: SHA1
            // mismatch" rather than only the outermost context.
            let detail = format!("{e:#}");
            tracing::error!(%server_id, error = %detail, "sync failed");
            modal.fail(detail.clone());
            ctx.send(MessageToFrontend::SyncFailed {
                server_id,
                reason: crate::failure::sync_failure_key(&e).into(),
                detail,
            });
            return;
        }
        ctx.send(MessageToFrontend::SyncComplete { server_id });
        // The files are in place, so the button has to stop offering to install
        // or update.
        ctx.send(MessageToFrontend::BuildStateChanged {
            server_id,
            state: crate::sync::build_state(&instance_dir, &manifest),
        });
        modal.finish();

        // Nothing looks at the directory between the sync and the launch, so
        // check it against the manifest here. Extra files go, mismatches go to
        // the master, and the player keeps launching: a finding is something to
        // look into later, not a refusal.
        let report =
            crate::sync::verify_before_launch(&instance_dir, &manifest, &enabled_optional, &user)
                .await;
        if !report.findings.is_empty() {
            tracing::warn!(findings = report.findings.len(), "found mismatched files");
        }
        // Only when something was actually put right. A new pack the player
        // added is a finding for the master, not a repair to announce.
        if report.findings.iter().any(|f| f.repaired) {
            ctx.send(MessageToFrontend::AddNotification {
                key: "notif-build-files-restored".into(),
                args: std::collections::BTreeMap::new(),
                level: schema::NotifLevel::Info,
            });
        }
        let blocked = report.block_launch;
        ctx.ws.send(ClientWsMsg::ReportIntegrity { report });
        if blocked {
            // The file is left where it is: the player has to see what is
            // holding them up, and deleting it quietly would look like the
            // launcher breaking.
            ctx.send(MessageToFrontend::AddNotification {
                key: "notif-launch-blocked".into(),
                args: std::collections::BTreeMap::new(),
                level: schema::NotifLevel::Error,
            });
            ctx.send(MessageToFrontend::SyncFailed {
                server_id,
                reason: crate::failure::LAUNCH_BLOCKED.into(),
                detail: String::new(),
            });
            return;
        }

        // Delivered packs are switched on the first time they arrive; the
        // network's prefix pack every time, chat is unreadable without it.
        crate::sync::live::enable_delivered_packs(
            &instance_dir,
            &manifest,
            &enabled_optional,
            &user,
        )
        .await;
        if instance_dir
            .join("resourcepacks/noro-prefixes.zip")
            .exists()
        {
            let _ = crate::sync::live::enable(&instance_dir, "noro-prefixes.zip").await;
        }

        // After the sync but before the launch: the game reads servers.dat at
        // start and rewrites it on exit. A broken server list is no reason to
        // keep the player out, so failures are only logged.
        if let Some(server) = &server {
            match crate::servers_dat::sync(&instance_dir, server) {
                Ok(true) => tracing::info!("servers.dat updated from the build's game servers"),
                Ok(false) => {}
                Err(e) => tracing::warn!("servers.dat not updated: {e}"),
            }
        }

        let launch_config = ctx
            .config
            .get()
            .launch_config_for_server(&server_id, &manifest.recommended_client_settings);
        let server_name = server
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "Minecraft".into());
        let online = server.as_ref().and_then(|s| s.online);
        let max_online = server.as_ref().and_then(|s| s.max_online);

        // Cancelled while the files were being checked: stop before the game.
        if modal.is_cancelled() {
            ctx.send(MessageToFrontend::LaunchCancelled { server_id });
            return;
        }

        // The channel to the case mod has to be up before the game starts: the
        // mod reads the handshake file once, at startup, and being late here
        // means no panel until the next login.
        ctx.mod_link.start(&ctx, instance_dir.clone()).await;

        match game_runner::launch(
            &ctx.http,
            &launch_config,
            &ctx.dirs,
            &server_id,
            &manifest,
            &login,
            connect,
        )
        .await
        {
            Ok(child) => {
                run_game_process(ctx, server_id, server_name, online, max_online, child).await;
            }
            Err(e) => {
                ctx.mod_link.stop().await;
                tracing::error!(%server_id, error = %format!("{e:#}"), "launch failed");
                let key = match crate::failure::sync_failure_key(&e) {
                    "sync-error-unknown" => crate::failure::LAUNCH_FAILED,
                    key => key,
                };
                ctx.send(MessageToFrontend::SyncFailed {
                    server_id,
                    reason: key.into(),
                    detail: format!("{e:#}"),
                });
            }
        }
    });
}

async fn run_game_process(
    ctx: Ctx,
    server_id: Uuid,
    server_name: String,
    online: Option<u32>,
    max_online: Option<u32>,
    mut child: tokio::process::Child,
) {
    let started = Instant::now();
    let start_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let (kill_tx, mut kill_rx) = mpsc::unbounded_channel::<()>();
    ctx.running.lock().insert(
        server_id,
        RunningGame {
            started,
            kill: kill_tx,
        },
    );

    ctx.rpc
        .update(crate::discord_rpc::DiscordRpcState::GameMenu {
            server_name: server_name.clone(),
            start_timestamp,
        });

    ctx.send(MessageToFrontend::GameStarted { server_id });
    ctx.ws.send(ClientWsMsg::ReportGameStart { server_id });

    if let Some(stdout) = child.stdout.take() {
        tokio::spawn(crate::log_reader::spawn_log_reader(
            stdout,
            server_id,
            ctx.frontend.clone(),
            false,
            Some(crate::log_reader::RpcLogContext {
                rpc: ctx.rpc.clone(),
                server_name: server_name.clone(),
                start_timestamp,
                online_current: online,
                online_max: max_online,
            }),
        ));
    }
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(crate::log_reader::spawn_log_reader(
            stderr,
            server_id,
            ctx.frontend.clone(),
            true,
            None,
        ));
    }

    let exit_ok = tokio::select! {
        status = child.wait() => status.map(|s| s.success()).unwrap_or(false),
        _ = kill_rx.recv() => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            false
        }
    };

    // The game is gone, so the channel comes down with it: a handshake file
    // left on disk promises access that no longer exists.
    ctx.mod_link.stop().await;

    let playtime = started.elapsed().as_secs();
    ctx.running.lock().remove(&server_id);
    ctx.ws.send(ClientWsMsg::ReportGameStop {
        server_id,
        playtime_secs: playtime,
    });
    ctx.send(MessageToFrontend::GameStopped { server_id, exit_ok });

    ctx.rpc
        .update(crate::discord_rpc::DiscordRpcState::Launcher {
            server_name: Some(server_name),
        });
}
