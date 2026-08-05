//! Ядро backend: состояние, главный событийный цикл, запуск игры.

use crate::auth::token_store;
use crate::config::{LauncherConfig, OptionalModsSelection};
use crate::directories::LauncherDirectories;
use crate::game_runner::{self, LoginInfo, ServerConnect};
use crate::persistent::Persistent;
use crate::ws_client::{self, WsClient};
use bridge::{BackendReceiver, FrontendHandle, MessageToBackend, MessageToFrontend, QuitHandler};
use parking_lot::Mutex;
use schema::{BuildManifest, ClientWsMsg, ServerEntry, ServerWsMsg, UserProfile};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

/// Запущенная игра.
pub struct RunningGame {
    pub started: Instant,
    /// Послать сюда → процесс будет убит.
    pub kill: UnboundedSender<()>,
}

/// Событие от фоновой задачи к главному циклу (нужно изменить состояние).
pub enum InternalEvent {
    LoginCompleted {
        auth: token_store::StoredAuth,
        user: UserProfile,
    },
    LoginFailed {
        kind: bridge::LoginErrorKind,
    },
    /// Установлено обновление — перезапуститься из нового бинарника.
    RestartInto(std::path::PathBuf),
    /// Skin/cape/profile change from native upload or external — refresh UI.
    ProfileUpdated {
        user: UserProfile,
    },
}

/// Клонируемый контекст, доступный фоновым задачам (sync/launch).
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
}

impl Ctx {
    pub fn send(&self, msg: MessageToFrontend) {
        self.frontend.send(msg);
    }
}

/// Полное состояние backend (живёт в главном цикле).
pub struct BackendState {
    pub ctx: Ctx,
    /// Команды от frontend.
    pub rx_backend: BackendReceiver,
    /// Координатор завершения.
    pub quit: QuitHandler,
    /// Входящие сообщения от мастера.
    pub master_rx: UnboundedReceiver<ServerWsMsg>,
    /// Состояние соединения.
    pub conn_rx: UnboundedReceiver<bool>,
    /// События от фоновых задач.
    pub internal_rx: UnboundedReceiver<InternalEvent>,

    // Кэши.
    pub user: Option<UserProfile>,
    pub access_token: Option<String>,
    pub servers: Vec<ServerEntry>,
    pub manifests: HashMap<Uuid, BuildManifest>,
    /// Сборки, ожидающие запуска после получения манифеста.
    pub pending_launch: HashMap<Uuid, bridge::ModalAction>,
}

const STARTUP_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Запустить backend на заданном рантайме. Не блокирует — спавнит главный цикл.
pub fn start(
    runtime: &tokio::runtime::Runtime,
    tx_frontend: FrontendHandle,
    rx_backend: BackendReceiver,
    quit: QuitHandler,
) {
    runtime.spawn(async move {
        if let Err(e) = run(tx_frontend, rx_backend, quit).await {
            tracing::error!("backend завершился с ошибкой: {e:#}");
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
            tracing::info!("автоматическая миграция: localhost -> 127.0.0.1 в конфиге");
        }
    });
    let optional = Persistent::<OptionalModsSelection>::load(dirs.optional_mods_file());
    let http = reqwest::Client::builder()
        .user_agent(format!("noro-launcher/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

    // Восстановить сессию из keyring.
    let stored = token_store::load();
    if stored.is_some() {
        tracing::info!("сессия загружена из keyring");
    } else {
        tracing::info!("сессия не найдена в keyring");
    }
    let access_token = stored.as_ref().map(|s| s.access_token.clone());

    // Каналы ws.
    let (master_tx, master_rx) = mpsc::unbounded_channel::<ServerWsMsg>();
    let (conn_tx, conn_rx) = mpsc::unbounded_channel::<bool>();
    let (internal_tx, internal_rx) = mpsc::unbounded_channel::<InternalEvent>();
    let ws = ws_client::spawn(
        config.get().ws_url(),
        access_token.clone(),
        master_tx,
        conn_tx,
    );

    tracing::info!("используется мастер-сервер: {}", config.get().master_url);

    let ctx = Ctx {
        frontend: tx_frontend,
        ws,
        http,
        dirs,
        config,
        optional,
        running: Arc::new(Mutex::new(HashMap::new())),
        internal: internal_tx,
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
        servers: Vec::new(),
        manifests: HashMap::new(),
        pending_launch: HashMap::new(),
    };

    // Если есть токен — подтянуть профиль через REST для мгновенного состояния входа.
    if state.access_token.is_some() {
        state.restore_session().await;
    }

    // Отдать конфиг и проверить обновление лаунчера при старте.
    state.send_config_state();
    // Каталог языка — из кеша сразу, с мастера следом.
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
                        // При (пере)подключении обновим списки.
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
        tracing::info!("backend: главный цикл завершён");
        // Отметиться в координаторе завершения.
        self.quit.clone().quit();
    }

    /// Обработать событие фоновой задачи.
    fn handle_internal(&mut self, event: InternalEvent) {
        match event {
            InternalEvent::LoginCompleted { auth, user } => {
                if let Err(e) = token_store::save(&auth) {
                    tracing::error!("не удалось сохранить сессию в keyring: {e}");
                } else {
                    tracing::info!("сессия сохранена в keyring");
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
        }
    }

    /// Login-инфо для запуска игры.
    pub fn login_info(&self) -> Option<LoginInfo> {
        let user = self.user.as_ref()?;
        let token = self.access_token.clone()?;
        Some(LoginInfo {
            username: user.username.clone(),
            uuid: user.uuid.simple().to_string(),
            access_token: token,
        })
    }

    /// Подключение к серверу для автоконнекта.
    pub fn server_connect(&self, server_id: &Uuid) -> Option<ServerConnect> {
        self.servers
            .iter()
            .find(|s| &s.id == server_id)
            .map(|s| ServerConnect {
                host: s.mc_host.clone(),
                port: s.mc_port,
            })
    }

    /// Восстановить профиль по сохранённому токену (REST /auth/me).
    async fn restore_session(&mut self) {
        let url = format!(
            "{}/auth/me",
            self.ctx.config.get().master_url.trim_end_matches('/')
        );
        let Some(token) = &self.access_token else {
            tracing::info!("restore_session: токен отсутствует");
            return;
        };
        tracing::info!("restore_session: попытка восстановить сессию через {url}");
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
                    tracing::info!(
                        "restore_session: сессия успешно восстановлена для {}",
                        profile.username
                    );
                    self.user = Some(profile.clone());
                    self.ctx
                        .send(MessageToFrontend::LoginSuccess { user: profile });
                } else {
                    tracing::warn!("restore_session: не удалось распарсить UserProfile");
                }
            }
            Ok(r) if r.status().as_u16() == 401 || r.status().as_u16() == 403 => {
                tracing::info!(
                    "restore_session: токен истёк ({}), пробуем refresh",
                    r.status()
                );
                self.try_refresh().await;
            }
            Ok(r) => {
                tracing::warn!("restore_session: сервер вернул статус {}", r.status());
            }
            Err(e) if e.is_connect() || e.is_timeout() => {
                tracing::error!("restore_session: сервер недоступен: {e}. Сессия сохранена.");
                // Не сбрасываем сессию, просто ждем восстановления связи.
            }
            Err(e) => {
                tracing::error!("restore_session: критическая ошибка запроса: {e}");
            }
        }
    }

    /// Обновить access-токен по refresh-токену.
    async fn try_refresh(&mut self) {
        let Some(stored) = token_store::load() else {
            tracing::info!("try_refresh: refresh_token не найден в keyring");
            return;
        };
        tracing::info!("try_refresh: попытка обновления токена");
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
                        tracing::info!("try_refresh: токен успешно обновлен");
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
                tracing::warn!("try_refresh: не удалось получить токены из ответа");
            } else {
                tracing::warn!("try_refresh: сервер вернул ошибку {}", r.status());
            }
        } else if let Err(e) = resp {
            tracing::error!("try_refresh: ошибка запроса: {e}");
        }
    }

    async fn restore_session_no_refresh(&mut self) {
        let url = format!(
            "{}/auth/me",
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

    /// Проверить доступное обновление лаунчера (REST).
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
                        // Мастер отдаёт git-тег («launcher-v1.2.0»), а у нас на
                        // руках версия крейта («1.2.0»): без снятия префикса они
                        // не совпадали никогда, и плашка обновления висела всегда.
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

/// Достроить объект LauncherVersion из ответа /api/launcher/version.
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

/// Запустить синхронизацию и игру в фоне.
pub fn spawn_sync_and_launch(
    ctx: Ctx,
    server_id: Uuid,
    manifest: BuildManifest,
    user: UserProfile,
    login: LoginInfo,
    connect: Option<ServerConnect>,
    enabled_optional: Vec<String>,
    // Карточка сборки — её игровые серверы уедут в servers.dat инстанса.
    server: Option<ServerEntry>,
    modal: bridge::ModalAction,
) {
    tokio::spawn(async move {
        let instance_dir = ctx.dirs.instance(&server_id);

        // Прогресс синхронизации → frontend + модалка.
        let to_fe = ctx.frontend.clone();
        let modal_clone = modal.clone();
        let progress: crate::sync::ProgressFn = Arc::new(move |stage, done, total, file| {
            to_fe.send(MessageToFrontend::SyncProgress {
                server_id,
                stage,
                done,
                total,
                file: file.clone(),
            });
            modal_clone.set_stage(stage.label());
            modal_clone.set_progress(done, total);
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
                run_game_process(ctx, server_id, child).await;
            }
            Err(e) => {
                ctx.send(MessageToFrontend::SyncFailed {
                    server_id,
                    reason: format!("запуск не удался: {e}"),
                });
            }
        }
    });
}

/// Управлять запущенным процессом: логи, ожидание, kill.
async fn run_game_process(ctx: Ctx, server_id: Uuid, mut child: tokio::process::Child) {
    let started = Instant::now();
    let (kill_tx, mut kill_rx) = mpsc::unbounded_channel::<()>();
    ctx.running.lock().insert(
        server_id,
        RunningGame {
            started,
            kill: kill_tx,
        },
    );

    ctx.send(MessageToFrontend::GameStarted { server_id });
    ctx.ws.send(ClientWsMsg::ReportGameStart { server_id });

    // Чтение stdout/stderr через новый log_reader.
    if let Some(stdout) = child.stdout.take() {
        tokio::spawn(crate::log_reader::spawn_log_reader(
            stdout,
            server_id,
            ctx.frontend.clone(),
            false,
        ));
    }
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(crate::log_reader::spawn_log_reader(
            stderr,
            server_id,
            ctx.frontend.clone(),
            true,
        ));
    }

    // Ожидание выхода или kill.
    let exit_ok = tokio::select! {
        status = child.wait() => status.map(|s| s.success()).unwrap_or(false),
        _ = kill_rx.recv() => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            false
        }
    };

    let playtime = started.elapsed().as_secs();
    ctx.running.lock().remove(&server_id);
    ctx.ws.send(ClientWsMsg::ReportGameStop {
        server_id,
        playtime_secs: playtime,
    });
    ctx.send(MessageToFrontend::GameStopped { server_id, exit_ok });
}
