//! Состояние UI и обработка сообщений от backend.

use bridge::{
    BackendHandle, ClientSettingsState, GameLogLevel, LoginErrorKind, MessageToBackend,
    MessageToFrontend, OptionalModInfo, SyncStage,
};
use gpui::{
    px, AppContext, Context, Entity, Image, IntoElement, ListAlignment, ListState, RenderImage,
};

use schema::{LauncherVersion, NewsItem, NotifLevel, ServerEntry, UserProfile};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

/// Текущий экран.
#[derive(Clone, PartialEq)]
pub enum Page {
    Login,
    Servers,
    ServerDetail(Uuid),
    ServerMods(Uuid),
    ServerSettings(Uuid),
    News,
    NewsDetail(Uuid),
    Profile,
    Settings,
}

/// Состояние синхронизации/игры конкретного сервера.
#[derive(Default, Clone)]
pub struct SyncUiState {
    pub stage: String,
    pub detail: String,
    /// Байты по стадиям загрузки: они идут параллельно, и у каждой своя полоса.
    /// BTreeMap — чтобы порядок строк не зависел от того, кто отчитался первым.
    pub stages: std::collections::BTreeMap<SyncStage, (u64, u64)>,
    pub syncing: bool,
    pub failed: Option<String>,
    pub running: bool,
}

impl SyncUiState {
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

/// Собрать текст уведомления из ключа и аргументов, пришедших по мосту.
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

/// Тост-уведомление.
#[derive(Clone)]
pub struct Toast {
    pub text: String,
    pub level: NotifLevel,
}

/// Конфиг для экрана настроек.
#[derive(Clone)]
pub struct UiConfig {
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub jvm_flags: String,
    pub show_console_on_launch: bool,
    pub master_url: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            memory_min_mb: 2048,
            memory_max_mb: 4096,
            jvm_flags: String::new(),
            show_console_on_launch: true,
            master_url: String::new(),
        }
    }
}

#[derive(Clone)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: GameLogLevel,
    pub text: String,
}

/// Корневая сущность UI (GPUI Render).
pub struct LauncherUI {
    pub backend: BackendHandle,
    pub page: Page,
    pub user: Option<UserProfile>,
    pub skin_image: Option<Arc<Image>>,
    /// Текущий кадр превью. `RenderImage`, а не `Image`: рисуется синхронно.
    pub skin_preview: Option<Arc<RenderImage>>,
    pub skin_bytes: Option<Vec<u8>>,
    pub skin_url: Option<String>,
    /// Поворот фигуры в градусах.
    pub skin_yaw: f32,
    /// Фаза покачивания конечностей в `[0, 1)`. Своя, не завязана на поворот.
    pub skin_sway: f32,
    pub skin_loading: bool,
    pub skin_uploading: bool,
    pub skin_dragging: bool,
    /// Cursor x at the last drag sample, in window px.
    pub skin_drag_x: f32,
    pub skin_anim_running: bool,
    pub cape_bytes: Option<Vec<u8>>,
    pub cape_url: Option<String>,
    pub cape_loading: bool,
    pub avatar_image: Option<Arc<Image>>,
    pub avatar_loading: bool,
    /// Язык интерфейса. Сам каталог живёт в глобальном состоянии i18n.
    pub locale: i18n::Locale,
    pub online: bool,
    pub logging_in: bool,
    pub startup_checking: bool,
    pub login_error: Option<String>,

    pub servers: Vec<ServerEntry>,
    pub news: Vec<NewsItem>,
    pub sync: HashMap<Uuid, SyncUiState>,
    /// Что делать со сборкой: ставить, обновлять или запускать.
    pub build_state: HashMap<Uuid, bridge::BuildState>,
    pub logs: HashMap<Uuid, Vec<LogEntry>>,
    pub optional_mods: HashMap<Uuid, Vec<OptionalModInfo>>,
    pub background_images: HashMap<Uuid, Arc<Image>>,
    pub news_images: HashMap<Uuid, Arc<Image>>,
    news_images_loading: HashSet<Uuid>,
    pub server_icons: HashMap<Uuid, Arc<Image>>,
    pub optional_mod_icons: HashMap<String, Arc<Image>>,
    background_image_urls: HashMap<Uuid, String>,
    server_icon_urls: HashMap<Uuid, String>,
    background_loading: HashSet<Uuid>,
    icons_loading: HashSet<Uuid>,
    optional_mod_icons_loading: HashSet<String>,

    pub update_available: Option<LauncherVersion>,
    pub updating: bool,
    pub toast: Option<Toast>,
    pub config: UiConfig,
    pub server_settings: HashMap<Uuid, ClientSettingsState>,
    pub server_recommendations: HashMap<Uuid, ClientSettingsState>,
    pub console_window: Option<gpui::WindowHandle<ConsoleWindow>>,
}
pub struct ConsoleWindow {
    pub server_id: Uuid,
    pub logs: Vec<LogEntry>,
    pub list_state: ListState,
    pub show_info: bool,
    pub show_warn: bool,
    pub show_error: bool,
    pub search_query: String,
    pub status_message: String,
    pub copy_success: bool,
}

pub struct GlobalLauncherUI(pub Entity<LauncherUI>);
impl gpui::Global for GlobalLauncherUI {}

impl gpui::Render for ConsoleWindow {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::pages::game_console;
        game_console::console_window_body(self, cx)
    }
}

const MAX_LOG_LINES: usize = 500;
const CONSOLE_WINDOW_SIZE: (f32, f32) = (800., 500.);
const CONSOLE_WINDOW_MIN_SIZE: (f32, f32) = (720., 440.);

impl LauncherUI {
    pub fn new(backend: BackendHandle) -> Self {
        // Запросим контент сразу (ответ придёт, когда ws подключится).
        backend.send(MessageToBackend::RequestServerList);
        backend.send(MessageToBackend::RequestNews);
        Self {
            backend,
            page: Page::Login,
            user: None,
            skin_image: None,
            skin_preview: None,
            skin_bytes: None,
            skin_url: None,
            skin_yaw: 0.0,
            skin_sway: 0.0,
            skin_loading: false,
            skin_uploading: false,
            skin_dragging: false,
            skin_drag_x: 0.0,
            skin_anim_running: false,
            cape_bytes: None,
            cape_url: None,
            cape_loading: false,
            avatar_image: None,
            avatar_loading: false,
            locale: i18n::Locale::default(),
            online: false,
            logging_in: false,
            startup_checking: true,
            login_error: None,
            servers: Vec::new(),
            news: Vec::new(),
            sync: HashMap::new(),
            build_state: HashMap::new(),
            logs: HashMap::new(),
            optional_mods: HashMap::new(),
            background_images: HashMap::new(),
            news_images: HashMap::new(),
            news_images_loading: HashSet::new(),
            server_icons: HashMap::new(),
            optional_mod_icons: HashMap::new(),
            background_image_urls: HashMap::new(),
            server_icon_urls: HashMap::new(),
            background_loading: HashSet::new(),
            icons_loading: HashSet::new(),
            optional_mod_icons_loading: HashSet::new(),
            update_available: None,
            updating: false,
            toast: None,
            config: UiConfig::default(),
            server_settings: HashMap::new(),
            server_recommendations: HashMap::new(),
            console_window: None,
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
            Page::ServerDetail(id) | Page::ServerMods(id) | Page::ServerSettings(id) => Some(id),
            _ => self.servers.first().map(|s| s.id),
        }
    }

    pub fn server_client_settings(&self, server_id: Uuid) -> ClientSettingsState {
        self.server_settings
            .get(&server_id)
            .cloned()
            .or_else(|| self.server_recommendations.get(&server_id).cloned())
            .unwrap_or_else(|| ClientSettingsState {
                memory_min_mb: self.config.memory_min_mb,
                memory_max_mb: self.config.memory_max_mb,
                jvm_flags: self.config.jvm_flags.clone(),
                show_console_on_launch: self.config.show_console_on_launch,
            })
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
        if self.background_image_urls.get(&server_id) == Some(&url)
            && (self.background_images.contains_key(&server_id)
                || self.background_loading.contains(&server_id))
        {
            return;
        }

        self.background_images.remove(&server_id);
        self.background_image_urls.insert(server_id, url.clone());
        self.background_loading.insert(server_id);
        cx.spawn(async move |this, cx| {
            let expected_url = url.clone();
            let result = crate::image_loader::load_image_from_url(url).await;
            let _ = this.update(cx, |state, cx| {
                state.background_loading.remove(&server_id);
                if state.background_image_urls.get(&server_id) != Some(&expected_url) {
                    return;
                }
                match result {
                    Ok(image) => {
                        state.background_images.insert(server_id, image);
                    }
                    Err(err) => {
                        let mut args = i18n::FluentArgs::new();
                        args.set("reason", err.to_string());
                        state.toast = Some(Toast {
                            text: i18n::t_args("error-background-failed", &args),
                            level: NotifLevel::Warning,
                        });
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
        if self.server_icon_urls.get(&server_id) == Some(&url)
            && (self.server_icons.contains_key(&server_id)
                || self.icons_loading.contains(&server_id))
        {
            return;
        }
        self.server_icons.remove(&server_id);
        self.server_icon_urls.insert(server_id, url.clone());
        self.icons_loading.insert(server_id);
        cx.spawn(async move |this, cx| {
            let expected_url = url.clone();
            let result = crate::image_loader::load_image_from_url(url).await;
            let _ = this.update(cx, |state, cx| {
                state.icons_loading.remove(&server_id);
                if state.server_icon_urls.get(&server_id) != Some(&expected_url) {
                    return;
                }
                if let Ok(image) = result {
                    state.server_icons.insert(server_id, image);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn ensure_optional_mod_icon_loaded(&mut self, url: Option<String>, cx: &mut Context<Self>) {
        let Some(url) = url.filter(|u| !u.trim().is_empty()) else {
            return;
        };
        if self.optional_mod_icons.contains_key(&url)
            || self.optional_mod_icons_loading.contains(&url)
        {
            return;
        }
        self.optional_mod_icons_loading.insert(url.clone());
        cx.spawn(async move |this, cx| {
            let result = crate::image_loader::load_image_from_url(url.clone()).await;
            let _ = this.update(cx, |state, cx| {
                state.optional_mod_icons_loading.remove(&url);
                if let Ok(image) = result {
                    state.optional_mod_icons.insert(url, image);
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn replace_servers(&mut self, servers: Vec<ServerEntry>) {
        let next_ids: HashSet<_> = servers.iter().map(|s| s.id).collect();
        let old_ids: Vec<_> = self.servers.iter().map(|s| s.id).collect();

        for id in old_ids {
            if !next_ids.contains(&id) {
                self.clear_server_assets(id);
            }
        }

        for server in &servers {
            self.sync_asset_url(server.id, server.background_url.as_ref(), true);
            self.sync_asset_url(server.id, server.icon_url.as_ref(), false);
        }

        self.servers = servers;
    }

    fn sync_asset_url(&mut self, server_id: Uuid, url: Option<&String>, is_background: bool) {
        let url = url.and_then(|u| {
            let trimmed = u.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        });
        match (is_background, url) {
            (true, Some(url)) if self.background_image_urls.get(&server_id) != Some(&url) => {
                self.background_images.remove(&server_id);
                self.background_loading.remove(&server_id);
                self.background_image_urls.insert(server_id, url);
            }
            (false, Some(url)) if self.server_icon_urls.get(&server_id) != Some(&url) => {
                self.server_icons.remove(&server_id);
                self.icons_loading.remove(&server_id);
                self.server_icon_urls.insert(server_id, url);
            }
            (true, None) => self.clear_background(server_id),
            (false, None) => self.clear_icon(server_id),
            _ => {}
        }
    }

    fn clear_server_assets(&mut self, server_id: Uuid) {
        self.clear_background(server_id);
        self.clear_icon(server_id);
    }

    fn clear_background(&mut self, server_id: Uuid) {
        self.background_images.remove(&server_id);
        self.background_loading.remove(&server_id);
        self.background_image_urls.remove(&server_id);
    }

    fn clear_icon(&mut self, server_id: Uuid) {
        self.server_icons.remove(&server_id);
        self.icons_loading.remove(&server_id);
        self.server_icon_urls.remove(&server_id);
    }

    /// Обработать сообщение от backend.
    pub fn on_message(&mut self, msg: MessageToFrontend, cx: &mut Context<Self>) {
        match msg {
            MessageToFrontend::LoginSuccess { user } => {
                self.user = Some(user);
                self.load_user_skin(cx);
                self.logging_in = false;
                self.startup_checking = false;
                self.login_error = None;
                if self.page == Page::Login {
                    self.page = Page::Servers;
                }
            }
            MessageToFrontend::LoginFailed { kind } => {
                self.logging_in = false;
                self.startup_checking = false;
                self.login_error = Some(match kind {
                    LoginErrorKind::Cancelled => i18n::t("error-sign-in-cancelled"),
                    // `r` — ключ перевода от мастера, а не готовый текст.
                    LoginErrorKind::Rejected(r) => i18n::t(&r),
                    LoginErrorKind::Network(e) => format!("Network: {e}"),
                });
            }
            MessageToFrontend::LoggedOut => {
                self.user = None;
                self.reset_skin_preview();
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
            MessageToFrontend::ServerList { servers } => self.replace_servers(servers),
            MessageToFrontend::NewsUpdated { items } => {
                self.news_images
                    .retain(|id, _| items.iter().any(|n| n.id == *id));
                self.news = items;
            }
            MessageToFrontend::ConfigState {
                memory_min_mb,
                memory_max_mb,
                jvm_flags,
                show_console_on_launch,
                master_url,
                locale,
                server_settings,
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
                    master_url,
                };
                self.server_settings = server_settings.into_iter().collect();
                if self.user.is_none() {
                    self.startup_checking = false;
                }
            }
            MessageToFrontend::LocaleCatalog { code, ftl } => {
                // Каталог с мастера перекрывает встроенный; битый — игнорируем.
                if let Some(loc) = i18n::Locale::from_code(&code) {
                    if loc == self.locale && !i18n::install_catalog(loc, &ftl) {
                        tracing::warn!("каталог перевода с мастера не разобрался");
                    }
                }
            }

            MessageToFrontend::OptionalMods { server_id, mods } => {
                self.optional_mods.insert(server_id, mods);
            }
            MessageToFrontend::ServerClientRecommendation {
                server_id,
                settings,
            } => {
                self.server_recommendations.insert(server_id, settings);
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
                if stage.is_download() {
                    s.stages.insert(stage, (done, total));
                    // Стадий в работе несколько — называть заголовком одну из
                    // них значило бы врать про остальные.
                    s.stage = "Downloading...".into();
                } else {
                    // Проверка файлов открывает новый прогон: полосы прошлого
                    // запуска к нему не относятся.
                    if stage == SyncStage::CheckingFiles && done == 0 {
                        s.stages.clear();
                    }
                    s.stage = stage.label().to_string();
                }
                if !file.is_empty() {
                    s.detail = file;
                }
                s.failed = None;
            }
            MessageToFrontend::SyncComplete { server_id } => {
                let s = self.sync.entry(server_id).or_default();
                s.syncing = false;
                s.stage = "Launching...".into();
            }
            MessageToFrontend::SyncFailed { server_id, reason } => {
                let s = self.sync.entry(server_id).or_default();
                s.syncing = false;
                s.failed = Some(reason.clone());
                self.toast = Some(Toast {
                    text: reason,
                    level: NotifLevel::Error,
                });
            }
            MessageToFrontend::GameStarted { server_id } => {
                let s = self.sync.entry(server_id).or_default();
                s.running = true;
                s.syncing = false;
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
                    self.toast = Some(Toast {
                        text: i18n::t("error-game-exited"),
                        level: NotifLevel::Warning,
                    });
                }
            }
            MessageToFrontend::GameLog {
                server_id,
                line,
                level,
                timestamp,
            } => {
                let logs = self.logs.entry(server_id).or_default();
                logs.push(LogEntry {
                    timestamp,
                    level,
                    text: line.clone(),
                });
                if logs.len() > MAX_LOG_LINES {
                    let drain = logs.len() - MAX_LOG_LINES;
                    logs.drain(0..drain);
                }

                // Обновить открытую консоль, если она есть.
                if let Some(handle) = &self.console_window {
                    let _ = handle.update(cx, |view, _, cx| {
                        if view.server_id == server_id {
                            view.logs.push(LogEntry {
                                timestamp,
                                level,
                                text: line,
                            });
                            if view.logs.len() > MAX_LOG_LINES {
                                let drain = view.logs.len() - MAX_LOG_LINES;
                                view.logs.drain(0..drain);
                            }

                            // Вычисляем количество элементов с учетом текущих фильтров
                            use crate::console_model::filtered_logs;
                            let visible_count = filtered_logs(
                                &view.logs,
                                view.show_info,
                                view.show_warn,
                                view.show_error,
                                &view.search_query,
                            )
                            .len();

                            view.list_state =
                                ListState::new(visible_count, ListAlignment::Bottom, px(100.));
                            cx.notify();
                        }
                    });
                }
            }
            MessageToFrontend::BuildStateChanged { server_id, state } => {
                self.build_state.insert(server_id, state);
            }
            MessageToFrontend::LauncherUpdateAvailable { version } => {
                self.update_available = Some(version);
            }
            MessageToFrontend::AddNotification { key, args, level } => {
                self.toast = Some(Toast {
                    text: translate_notification(&key, &args),
                    level,
                });
            }
            MessageToFrontend::SkinUploadFailed => {
                self.skin_uploading = false;
            }
            MessageToFrontend::PermissionsUpdated { user } => {
                self.user = Some(user);
                self.load_user_skin(cx);
            }
            MessageToFrontend::ConnectionState { online } => {
                self.online = online;
            }
            MessageToFrontend::OpenOrFocusMainWindow => {}
            MessageToFrontend::CloseModal => {
                self.logging_in = false;
            }
            MessageToFrontend::Quit => {
                cx.quit();
            }
        }
        cx.notify();
    }

    // --- Действия из UI ---

    pub fn start_login(&mut self) {
        self.logging_in = true;
        self.login_error = None;
        let modal = bridge::ModalAction::new("Discord sign in");
        self.backend.send(MessageToBackend::StartDiscordLogin {
            modal_action: modal,
        });
    }

    pub fn logout(&mut self) {
        self.backend.send(MessageToBackend::Logout);
    }

    pub fn upload_skin(&mut self, bytes: Vec<u8>) {
        if self.skin_uploading {
            return;
        }
        // Отдельный флаг: skin_loading занят скачиванием текстуры, и держать его
        // здесь означало бы заблокировать загрузку только что залитого скина.
        self.skin_uploading = true;
        self.backend.send(MessageToBackend::UploadSkin { bytes });
    }

    pub fn open_news(&mut self, id: Uuid, cx: &mut Context<Self>) {
        self.page = Page::NewsDetail(id);
        self.load_news_image(id, cx);
    }

    /// Картинка новости тянется лениво — на списке она не нужна, а новостей
    /// может быть много.
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
            let result = crate::image_loader::load_image_from_url(url).await;
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
        let s = self.sync.entry(id).or_default();
        s.syncing = true;
        s.failed = None;
        s.stage = "Preparing...".into();
        let modal = bridge::ModalAction::new("Launch");
        self.backend.send(MessageToBackend::LaunchServer {
            server_id: id,
            modal_action: modal,
        });
    }

    pub fn kill(&mut self, id: Uuid) {
        self.backend
            .send(MessageToBackend::KillGame { server_id: id });
    }

    pub fn toggle_optional(&mut self, server_id: Uuid, name: &str) {
        if let Some(mods) = self.optional_mods.get_mut(&server_id) {
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

    pub fn set_server_show_console_on_launch(&mut self, server_id: Uuid, enabled: bool) {
        let mut settings = self.server_client_settings(server_id);
        settings.show_console_on_launch = enabled;
        self.server_settings.insert(server_id, settings);
        self.backend
            .send(MessageToBackend::SetServerShowConsoleOnLaunch { server_id, enabled });
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

    /// Переключить язык интерфейса. Каталог с мастера подтянет backend.
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

    pub fn install_update(&mut self) {
        if let Some(v) = self.update_available.clone() {
            self.updating = true;
            let modal = bridge::ModalAction::new("Update");
            self.backend.send(MessageToBackend::InstallUpdate {
                version: v,
                modal_action: modal,
            });
        }
    }

    pub fn toggle_console(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_server_id() {
            self.open_console(id, cx);
        }
    }

    pub fn open_console(&mut self, server_id: Uuid, cx: &mut Context<Self>) {
        if let Some(handle) = &self.console_window {
            let _ = handle.update(cx, |_, _, cx| {
                cx.notify();
            });
            return;
        }

        let bounds = gpui::Bounds::centered(
            None,
            gpui::size(px(CONSOLE_WINDOW_SIZE.0), px(CONSOLE_WINDOW_SIZE.1)),
            cx,
        );
        let logs = self.logs.get(&server_id).cloned().unwrap_or_default();
        let handle = cx.open_window(
            gpui::WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                window_min_size: Some(gpui::size(
                    px(CONSOLE_WINDOW_MIN_SIZE.0),
                    px(CONSOLE_WINDOW_MIN_SIZE.1),
                )),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some(gpui::SharedString::new_static("Noro Game Console")),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |_, cx| {
                cx.new(|cx| {
                    cx.on_release(|_: &mut ConsoleWindow, cx| {
                        if let Some(ui) = cx.try_global::<GlobalLauncherUI>() {
                            let ui = ui.0.clone();
                            let _ = ui.update(cx, |this_ui, cx| {
                                this_ui.console_window = None;
                                cx.notify();
                            });
                        }
                    })
                    .detach();
                    ConsoleWindow {
                        server_id,
                        logs: logs.clone(),
                        list_state: ListState::new(logs.len(), ListAlignment::Bottom, px(100.)),
                        show_info: true,
                        show_warn: true,
                        show_error: true,
                        search_query: String::new(),
                        status_message: String::new(),
                        copy_success: false,
                    }
                })
            },
        );

        if let Ok(h) = handle {
            self.console_window = Some(h);
        }
    }
}
