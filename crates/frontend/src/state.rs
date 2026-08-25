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
    ServerModCatalog(Uuid),
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
    pub fullscreen: bool,
    pub crash_reports: bool,
    /// Вшит ли DSN в сборку. Нет — строку настройки не показываем: переключать
    /// было бы нечего, а обещание «мы это шлём» оказалось бы ложным.
    pub crash_reports_available: bool,
    pub master_url: String,
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

/// Вкладка страницы профиля.
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
    pub bytes: Vec<u8>,
    pub preview: Option<Arc<RenderImage>>,
}

/// Корневая сущность UI (GPUI Render).
pub struct LauncherUI {
    pub backend: BackendHandle,
    pub page: Page,
    pub profile_tab: ProfileTab,
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
    pub capes: Vec<schema::CapeRow>,
    pub cape_images: std::collections::HashMap<uuid::Uuid, std::sync::Arc<gpui::Image>>,
    pub preset_images: std::collections::HashMap<String, std::sync::Arc<gpui::Image>>,
    pub custom_presets: Vec<SavedSkinPreset>,
    pub cape_selector_open: bool,
    /// Раскрыт ли список версий сборки в нижней панели.
    pub build_picker_open: bool,
    pub avatar_image: Option<Arc<Image>>,
    pub avatar_loading: bool,
    /// Язык интерфейса. Сам каталог живёт в глобальном состоянии i18n.
    pub locale: i18n::Locale,
    pub online: bool,
    pub logging_in: bool,
    pub sidebar_collapsed: bool,
    pub mod_catalog_hits: Vec<bridge::CatalogHitInfo>,
    pub mod_catalog_selected: Option<bridge::CatalogHitInfo>,
    pub mod_catalog_provider: String,
    pub mod_catalog_query: String,
    pub mod_catalog_focus: Option<gpui::FocusHandle>,
    /// Страница выбранного мода: описание, скриншоты, ссылки.
    pub mod_project: Option<bridge::ModProjectInfo>,
    pub mod_detail_gallery: bool,
    pub mod_catalog_total: u32,
    pub mod_catalog_offset: u32,
    pub mod_catalog_limit: u32,
    /// Почему каталог пуст. `None` — либо ещё ищем, либо всё в порядке.
    pub mod_catalog_error: Option<String>,
    /// Переименование пресета скина: id и черновик имени. Правится прямо в
    /// карточке — системного диалога ввода текста нет ни на одной платформе.
    pub renaming_preset: Option<(String, String)>,
    pub rename_focus: Option<gpui::FocusHandle>,
    pub startup_checking: bool,
    pub login_error: Option<String>,
    pub login_mode_key: bool,
    pub login_key_input: String,
    pub login_key_focus: Option<gpui::FocusHandle>,

    pub servers: Vec<ServerEntry>,
    /// Версия, выбранная игроком по серверам. Нет записи — текущая.
    pub selected_build: std::collections::HashMap<Uuid, Option<Uuid>>,
    pub news: Vec<NewsItem>,
    pub sync: HashMap<Uuid, SyncUiState>,
    /// Что делать со сборкой: ставить, обновлять или запускать.
    pub build_state: HashMap<Uuid, bridge::BuildState>,
    pub logs: HashMap<Uuid, Vec<LogEntry>>,
    pub optional_mods: HashMap<Uuid, Vec<OptionalModInfo>>,
    pub installed_files: HashMap<Uuid, Vec<String>>,
    pub allow_mod_suggestions: HashMap<Uuid, bool>,
    pub suggested_mods: HashSet<String>,
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
    /// Запрос на вход в чужой аккаунт, ждущий подтверждения.
    pub impersonate_prompt: Option<ImpersonatePrompt>,
    /// Ник игрока, от чьего имени сейчас работает лаунчер.
    pub impersonating_as: Option<String>,
    /// Запрос логов, ждущий решения.
    pub log_request_prompt: Option<LogRequestPrompt>,
    /// Открыт ли предпросмотр того, что уйдёт.
    pub log_request_preview_open: bool,
    /// Предложенное админом действие, ждущее решения.
    pub remote_action_prompt: Option<RemoteActionPrompt>,
}

/// Действие, о котором просит админ.
pub struct RemoteActionPrompt {
    pub action: schema::RemoteAction,
    pub server_id: Option<Uuid>,
    pub actor_username: String,
}

/// Запрос логов от админа.
pub struct LogRequestPrompt {
    pub request_id: Uuid,
    pub actor_username: String,
    pub reason: String,
    /// Собран без спроса: логи уже уехали, модалка только сообщает.
    pub forced: bool,
    pub preview: String,
    pub files: Vec<(String, u64)>,
}

/// Диалог «войти в аккаунт игрока».
pub struct ImpersonatePrompt {
    pub grant_id: Uuid,
    pub target_username: String,
    pub reason: String,
    pub expires_in_secs: i64,
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
            skin_dragging: false,
            skin_drag_x: 0.0,
            skin_anim_running: false,
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
            locale: i18n::Locale::default(),
            online: false,
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
            allow_mod_suggestions: HashMap::new(),
            suggested_mods: HashSet::new(),
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
            impersonate_prompt: None,
            log_request_prompt: None,
            log_request_preview_open: false,
            remote_action_prompt: None,
            impersonating_as: None,
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
            Page::ServerDetail(id)
            | Page::ServerMods(id)
            | Page::ServerModCatalog(id)
            | Page::ServerSettings(id) => Some(id),
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
                fullscreen: self.config.fullscreen,
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

    pub fn save_current_skin_preset(&mut self) {
        if let Some(bytes) = &self.skin_bytes {
            let num = self.custom_presets.len() + 1;
            let name = format!("Смерч {}", num);
            let id = uuid::Uuid::new_v4().to_string();
            let preset = SavedSkinPreset {
                id,
                name,
                bytes: bytes.clone(),
                preview: self.skin_preview.clone(),
            };
            self.custom_presets.push(preset);
        }
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

    /// Обработать сообщение от backend.
    pub fn on_message(&mut self, msg: MessageToFrontend, cx: &mut Context<Self>) {
        match msg {
            MessageToFrontend::LoginSuccess { user } => {
                self.user = Some(user);
                self.load_user_skin(cx);
                self.logging_in = false;
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
                fullscreen,
                crash_reports,
                crash_reports_available,
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
                    fullscreen,
                    crash_reports,
                    crash_reports_available,
                    master_url,
                };
                self.server_settings = server_settings.into_iter().collect();
                self.load_preset_renders(cx);
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

            MessageToFrontend::OptionalMods {
                server_id,
                mods,
                allow_suggestions,
                installed_files,
            } => {
                self.optional_mods.insert(server_id, mods);
                self.allow_mod_suggestions
                    .insert(server_id, allow_suggestions);
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
            }
            MessageToFrontend::CatalogFailed { message } => {
                self.mod_catalog_error = Some(message);
            }
            MessageToFrontend::ModProjectLoaded { project } => {
                // Ответ мог прийти после того, как игрок ушёл на другой мод.
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
            MessageToFrontend::LiveSynced {
                server_id,
                updated,
                locked,
            } => {
                // Игра запущена, и в ней появились новые паки. Само по себе это
                // не видно: клиенту нужна перезагрузка ресурсов, а решает это
                // игрок — посреди боя она некстати.
                let s = self.sync.entry(server_id).or_default();
                s.stage = if locked.is_empty() {
                    format!(
                        "Обновлено наборов: {}. Нажмите F3+T, чтобы применить",
                        updated.len()
                    )
                } else {
                    format!(
                        "Обновлено наборов: {}. Ещё {} встанут при следующем запуске",
                        updated.len(),
                        locked.len()
                    )
                };
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
            }
            MessageToFrontend::SkinUploadFailed => {
                self.skin_uploading = false;
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
                        bytes: Vec::new(),
                        preview: None,
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
                                    found.bytes = bytes;
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

    /// Вход через сайт: платформ много, и все они живут там.
    pub fn start_login(&mut self) {
        self.logging_in = true;
        self.login_error = None;
        let modal = bridge::ModalAction::new("Website sign in");
        self.backend.send(MessageToBackend::StartWebLogin {
            modal_action: modal,
        });
    }

    pub fn start_key_login(&mut self, key: String) {
        if key.trim().is_empty() {
            return;
        }
        self.logging_in = true;
        self.login_error = None;
        let modal = bridge::ModalAction::new("Key sign in");
        self.backend.send(MessageToBackend::StartKeyLogin {
            key: key.trim().to_string(),
            modal_action: modal,
        });
    }

    pub fn start_biometric_login(&mut self) {
        self.logging_in = true;
        self.login_error = None;
        let modal = bridge::ModalAction::new("Biometric sign in");
        self.backend.send(MessageToBackend::StartBiometricLogin {
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
        self.skin_bytes = Some(bytes.clone());
        self.skin_uploading = true;
        self.backend.send(MessageToBackend::UploadSkin { bytes });
    }

    /// Сменить модель скина. Картинка остаётся, меняется ширина рук; профиль
    /// приедет обратно тем же путём, что и после загрузки.
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

    /// Включить или выключить опциональный мод.
    ///
    /// Включение проверяется правилами сборки: несовместимый мод и мод без
    /// своей зависимости не включаются, а игрок получает причину. Молча гасить
    /// соседа нельзя — выбор между двумя несовместимыми модами его, а не наш.
    pub fn toggle_optional(&mut self, server_id: Uuid, name: &str) {
        if let Some(mods) = self.optional_mods.get_mut(&server_id) {
            let turning_on = mods
                .iter()
                .find(|m| m.name == name)
                .is_some_and(|m| !m.enabled);
            if turning_on {
                if let Some(issue) = Self::blocking_issue(mods, name) {
                    self.toast = Some(issue);
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

    /// Что мешает включить мод. `None` — включать можно.
    ///
    /// Правила общие с мастером (`schema::optional`): разъедься они, лаунчер
    /// разрешал бы то, что мастер отвергает.
    fn blocking_issue(mods: &[OptionalModInfo], name: &str) -> Option<Toast> {
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
        Some(Toast {
            text: i18n::t_args(key, &args),
            level: NotifLevel::Warning,
        })
    }

    /// Из того, что знает интерфейс, — в правило сборки. Интерфейсу хватает
    /// имени и связей: остальные поля правила на решение не влияют.
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

    /// Выбрать версию сборки для сервера.
    ///
    /// `None` — вернуться к текущей опубликованной. Выбор хранит бэкенд: он же
    /// перезапрашивает манифест, поэтому список файлов и модов обновится сам.
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

    /// Отправка отчётов о падениях. Вступает в силу со следующего запуска:
    /// Sentry поднимается до GPUI, и снять его хук паники на ходу нельзя.
    pub fn set_crash_reports(&mut self, enabled: bool) {
        self.config.crash_reports = enabled;
        self.backend
            .send(MessageToBackend::SetCrashReports { enabled });
    }

    /// «Сообщить о проблеме»: собрать логи текущего сервера и отправить.
    ///
    /// Сервер не указан — backend возьмёт тот, чей манифест уже загружен: логи
    /// лежат в каталоге инстанса, и без сервера отправлять нечего.
    /// Ответить на запрос логов.
    pub fn answer_log_request(&mut self, accepted: bool) {
        let Some(prompt) = self.log_request_prompt.take() else {
            return;
        };
        self.backend.send(MessageToBackend::LogRequestAnswer {
            request_id: prompt.request_id,
            accepted,
        });
    }

    /// Ответить на предложенное действие.
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

    /// Закрыть окно принудительного сбора: отвечать там нечего.
    pub fn dismiss_log_request(&mut self) {
        self.log_request_prompt = None;
    }

    /// Ответить на диалог входа в чужой аккаунт.
    pub fn answer_impersonate(&mut self, accepted: bool) {
        let Some(prompt) = self.impersonate_prompt.take() else {
            return;
        };
        self.backend.send(MessageToBackend::ImpersonateAnswer {
            grant_id: prompt.grant_id,
            accepted,
        });
    }

    /// Выйти из чужого аккаунта.
    pub fn exit_impersonation(&mut self) {
        self.backend.send(MessageToBackend::ImpersonateExit);
    }

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
                            ui.update(cx, |this_ui, cx| {
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
