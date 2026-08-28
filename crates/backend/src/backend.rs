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
use std::collections::{BTreeMap, HashMap};
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
}

const STARTUP_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

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
        if c.fix_localhost() {
            tracing::info!("migrated config: localhost -> 127.0.0.1");
        }
    });
    let optional = Persistent::<OptionalModsSelection>::load(dirs.optional_mods_file());
    let http = reqwest::Client::builder()
        .user_agent(format!("noro-launcher/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

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
    };

    // Over REST rather than waiting for the socket: the login screen would
    // otherwise flash by on every start.
    if state.access_token.is_some() {
        state.restore_session().await;
    }

    state.send_config_state();
    crate::translations::refresh(&state.ctx, state.ctx.config.get().locale);
    state.check_launcher_update().await;

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
        self.quit.clone().quit();
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
                self.ctx.send(MessageToFrontend::Quit);
                crate::updater::restart(&exe);
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
        }
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

    async fn restore_session(&mut self) {
        let url = format!(
            "{}/api/me",
            self.ctx.config.get().master_url.trim_end_matches('/')
        );
        let Some(token) = &self.access_token else {
            tracing::info!("restore_session: no token");
            return;
        };
        tracing::info!("restore_session: trying {url}");
        let resp = self
            .ctx
            .http
            .get(&url)
            .bearer_auth(token)
            .timeout(STARTUP_REQUEST_TIMEOUT)
            .send()
            .await;
        match resp {
            Ok(r) if r.status().is_success() => {
                if let Ok(profile) = r.json::<UserProfile>().await {
                    tracing::info!("restore_session: restored for {}", profile.username);
                    self.user = Some(profile.clone());
                    self.ctx
                        .send(MessageToFrontend::LoginSuccess { user: profile });
                } else {
                    tracing::warn!("restore_session: UserProfile did not parse");
                }
            }
            Ok(r) if r.status().as_u16() == 401 || r.status().as_u16() == 403 => {
                tracing::info!("restore_session: token expired ({}), refreshing", r.status());
                self.try_refresh().await;
            }
            Ok(r) => {
                tracing::warn!("restore_session: master returned {}", r.status());
            }
            // Unreachable is not the same as rejected: keep the session and
            // wait for the network rather than logging the player out.
            Err(e) if e.is_connect() || e.is_timeout() => {
                tracing::error!("restore_session: master unreachable: {e}, session kept");
            }
            Err(e) => {
                tracing::error!("restore_session: request failed: {e}");
            }
        }
    }

    async fn try_refresh(&mut self) {
        let Some(stored) = token_store::load() else {
            tracing::info!("try_refresh: no refresh_token in the keyring");
            return;
        };
        tracing::info!("try_refresh: refreshing the token");
        let url = format!(
            "{}/auth/refresh",
            self.ctx.config.get().master_url.trim_end_matches('/')
        );
        let resp = self
            .ctx
            .http
            .post(&url)
            .json(&serde_json::json!({ "refresh_token": stored.refresh_token }))
            .timeout(STARTUP_REQUEST_TIMEOUT)
            .send()
            .await;
        if let Ok(r) = resp {
            if r.status().is_success() {
                if let Ok(v) = r.json::<serde_json::Value>().await {
                    if let (Some(at), Some(rt)) =
                        (v["access_token"].as_str(), v["refresh_token"].as_str())
                    {
                        tracing::info!("try_refresh: token refreshed");
                        let _ = token_store::save(&token_store::StoredAuth {
                            access_token: at.to_string(),
                            refresh_token: rt.to_string(),
                        });
                        self.access_token = Some(at.to_string());
                        self.ctx.ws.set_token(Some(at.to_string()));
                        self.restore_session_no_refresh().await;
                        return;
                    }
                }
                tracing::warn!("try_refresh: response had no tokens in it");
            } else {
                tracing::warn!("try_refresh: master returned {}", r.status());
            }
        } else if let Err(e) = resp {
            tracing::error!("try_refresh: request failed: {e}");
        }
    }

    async fn restore_session_no_refresh(&mut self) {
        let url = format!(
            "{}/api/me",
            self.ctx.config.get().master_url.trim_end_matches('/')
        );
        if let Some(token) = self.access_token.clone() {
            if let Ok(r) = self
                .ctx
                .http
                .get(&url)
                .bearer_auth(&token)
                .timeout(STARTUP_REQUEST_TIMEOUT)
                .send()
                .await
            {
                if let Ok(profile) = r.json::<UserProfile>().await {
                    self.user = Some(profile.clone());
                    self.ctx
                        .send(MessageToFrontend::LoginSuccess { user: profile });
                }
            }
        }
    }

    async fn check_launcher_update(&self) {
        let url = format!(
            "{}/api/launcher/version?platform={}",
            self.ctx.config.get().master_url.trim_end_matches('/'),
            schema::current_platform()
        );
        if let Ok(r) = self.ctx.http.get(&url).send().await {
            if let Ok(v) = r.json::<serde_json::Value>().await {
                if !v.is_null() {
                    if let Some(version) = v["version"].as_str() {
                        // The master reports a git tag, "launcher-v1.2.0", and
                        // what we have is the crate version, "1.2.0". Without
                        // stripping the prefix they never match and the update
                        // banner is always up.
                        if version.trim_start_matches("launcher-v") != env!("CARGO_PKG_VERSION") {
                            if let Ok(lv) = serde_json::from_value::<schema::LauncherVersion>(
                                build_launcher_version(&v),
                            ) {
                                self.ctx.send(MessageToFrontend::LauncherUpdateAvailable {
                                    version: lv,
                                });
                            }
                        }
                    }
                }
            }
        }
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

/// Everything needed to sync a build and start the game.
pub struct Launch {
    pub ctx: Ctx,
    pub server_id: Uuid,
    pub manifest: BuildManifest,
    pub user: UserProfile,
    pub login: LoginInfo,
    pub connect: Option<ServerConnect>,
    pub enabled_optional: Vec<String>,
    /// Карточка сборки — её игровые серверы уедут в servers.dat инстанса.
    pub server: Option<ServerEntry>,
    pub modal: bridge::ModalAction,
}

/// Запустить синхронизацию и игру в фоне.
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

        // Прогресс синхронизации → frontend + модалка.
        let to_fe = ctx.frontend.clone();
        let modal_clone = modal.clone();
        // Стадии загрузки идут параллельно, а полоса в модалке одна. Держим
        // последний отчёт каждой стадии и показываем сумму — иначе полоса
        // скакала бы туда-сюда вслед за тем, чей отчёт пришёл последним.
        let totals: Arc<Mutex<BTreeMap<bridge::SyncStage, (u64, u64)>>> =
            Arc::new(Mutex::new(BTreeMap::new()));
        let progress: crate::sync::ProgressFn = Arc::new(move |stage, done, total, file| {
            to_fe.send(MessageToFrontend::SyncProgress {
                server_id,
                stage,
                done,
                total,
                file: file.clone(),
            });
            if stage.is_download() {
                let (sum_done, sum_total) = {
                    let mut g = totals.lock();
                    g.insert(stage, (done, total));
                    g.values()
                        .fold((0u64, 0u64), |(d, t), (sd, st)| (d + sd, t + st))
                };
                modal_clone.set_stage("Downloading...");
                modal_clone.set_progress(sum_done, sum_total);
            } else {
                modal_clone.set_stage(stage.label());
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
            modal.fail(e.to_string());
            ctx.send(MessageToFrontend::SyncFailed {
                server_id,
                reason: e.to_string(),
            });
            return;
        }
        ctx.send(MessageToFrontend::SyncComplete { server_id });
        // Файлы уже на месте — кнопка должна перестать звать ставить или обновлять.
        ctx.send(MessageToFrontend::BuildStateChanged {
            server_id,
            state: crate::sync::build_state(&instance_dir, &manifest),
        });
        modal.finish();

        // Сверка каталога с манифестом в последний момент: между синком и
        // запуском файлы никто не проверяет. Лишнее удаляется, расхождения
        // уезжают мастеру. Игрок при этом видит нейтральное сообщение и
        // продолжает запуск — флаг это повод для разбора, а не отказ.
        let report =
            crate::sync::verify_before_launch(&instance_dir, &manifest, &enabled_optional, &user)
                .await;
        if !report.findings.is_empty() {
            tracing::warn!(findings = report.findings.len(), "сверка нашла расхождения");
            ctx.send(MessageToFrontend::AddNotification {
                key: "notif-build-files-restored".into(),
                args: std::collections::BTreeMap::new(),
                level: schema::NotifLevel::Info,
            });
        }
        let blocked = report.block_launch;
        ctx.ws.send(ClientWsMsg::ReportIntegrity { report });
        if blocked {
            // Не удаляем сами: игрок должен увидеть, из-за чего его не пускают,
            // а удаление молча выглядело бы поломкой лаунчера.
            ctx.send(MessageToFrontend::AddNotification {
                key: "notif-launch-blocked".into(),
                args: std::collections::BTreeMap::new(),
                level: schema::NotifLevel::Error,
            });
            ctx.send(MessageToFrontend::SyncFailed {
                server_id,
                reason: "запуск заблокирован: найден запрещённый файл".into(),
            });
            return;
        }

        // После синхронизации файлов, но до запуска: игра читает servers.dat
        // на старте и перезаписывает его при выходе. Список серверов не повод
        // не пустить игрока, поэтому ошибку только логируем.
        if let Some(server) = &server {
            match crate::servers_dat::sync(&instance_dir, server) {
                Ok(true) => tracing::info!("servers.dat обновлён по игровым серверам сборки"),
                Ok(false) => {}
                Err(e) => tracing::warn!("servers.dat не обновлён: {e}"),
            }
        }

        // Запуск игры.
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

        // Канал с модом разбора поднимаем до запуска: мод читает файл
        // рукопожатия на старте игры, и опоздать здесь значит остаться без
        // панели до следующего входа.
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
                ctx.send(MessageToFrontend::SyncFailed {
                    server_id,
                    reason: format!("запуск не удался: {e}"),
                });
            }
        }
    });
}

/// Управлять запущенным процессом: логи, ожидание, kill.
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

    // Чтение stdout/stderr через новый log_reader.
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

    // Ожидание выхода или kill с активной проверкой PID каждые 500 мс.
    let mut poll_interval = tokio::time::interval(std::time::Duration::from_millis(500));
    let exit_ok = loop {
        tokio::select! {
            status = child.wait() => {
                break status.map(|s| s.success()).unwrap_or(false);
            }
            _ = kill_rx.recv() => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                break false;
            }
            _ = poll_interval.tick() => {
                match child.try_wait() {
                    Ok(Some(status)) => break status.success(),
                    Ok(None) => {}
                    Err(e) => {
                        tracing::error!("ошибка проверки статуса процесса игры: {e}");
                        break false;
                    }
                }
            }
        }
    };

    // Игра закрылась — гасим канал и убираем ключ: оставить файл рукопожатия
    // лежать значит обещать доступ, которого больше нет.
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
