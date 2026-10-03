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

/// Sync and run state for one server.
#[derive(Default, Clone)]
pub struct SyncUiState {
    pub stage: String,
    pub detail: String,
    /// Bytes per download stage — they run in parallel and each gets its own
    /// bar. BTreeMap so the row order doesn't depend on who reported first.
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
    /// Растёт на каждую плашку. Нужен, чтобы таймер снял именно свою: пока он
    /// спит, стопка успевает смениться целиком.
    pub id: u64,
    pub text: String,
    pub level: NotifLevel,
}

impl Toast {
    /// Сколько плашка живёт.
    ///
    /// Чем хуже новость, тем дольше: «скин загружен» читается краем глаза, а
    /// причину, по которой не запустилась игра, человек ещё и перечитывает.
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
    pub master_url: String,
    pub console: bridge::ConsoleSettings,
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
            console: bridge::ConsoleSettings::default(),
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
    pub bytes: Vec<u8>,
    pub preview: Option<Arc<RenderImage>>,
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
    pub skin_bytes: Option<Vec<u8>>,
    pub skin_url: Option<String>,
    /// Rotation of the figure, in degrees.
    pub skin_yaw: f32,
    /// Limb sway phase in `[0, 1)`. Its own clock, not tied to the rotation.
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
    pub build_picker_open: bool,
    pub avatar_image: Option<Arc<Image>>,
    pub avatar_loading: bool,
    /// Frame counters. Idle unless `NORO_PERF` is set.
    pub perf: crate::perf::Perf,
    /// UI language. The catalog itself lives in i18n's global state.
    pub locale: i18n::Locale,
    pub online: bool,
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
    pub login_mode_key: bool,
    pub login_key_input: String,
    pub login_key_focus: Option<gpui::FocusHandle>,

    pub servers: Vec<ServerEntry>,
    /// Build the player picked, per server. No entry means the current one.
    pub selected_build: std::collections::HashMap<Uuid, Option<Uuid>>,
    pub news: Vec<NewsItem>,
    pub sync: HashMap<Uuid, SyncUiState>,
    pub build_state: HashMap<Uuid, bridge::BuildState>,
    pub logs: HashMap<Uuid, Vec<LogEntry>>,
    pub optional_mods: HashMap<Uuid, Vec<OptionalModInfo>>,
    pub installed_files: HashMap<Uuid, Vec<String>>,
    /// Имена того, что сборка уже везёт, приведённые к виду для сравнения.
    ///
    /// Считается один раз на приход манифеста: карточка каталога спрашивает
    /// «это уже стоит?» на каждом кадре, и приводить к общему виду сотни имён
    /// каждый раз — это и есть тормоза списка.
    pub installed_keys: HashMap<Uuid, std::collections::HashSet<String>>,
    pub allow_mod_suggestions: HashMap<Uuid, bool>,
    /// Разрешает ли сборка свой контент. Вкладки «Моды» без этого нет вовсе:
    /// кнопка, ведущая к отказу, хуже её отсутствия.
    pub allow_personal_content: HashMap<Uuid, bool>,
    pub suggested_mods: HashSet<String>,
    pub background_images: HashMap<Uuid, Arc<RenderImage>>,
    pub news_images: HashMap<Uuid, Arc<Image>>,
    news_images_loading: HashSet<Uuid>,
    pub server_icons: HashMap<Uuid, Arc<RenderImage>>,
    /// Уже разобранные пиксели, а не сжатый файл: `Image` уходит в кеш ассетов
    /// GPUI и разбирается там при отрисовке, и на списке из двадцати иконок это
    /// стоило 311 мс на кадр против 30 мс на том же экране без картинок.
    pub optional_mod_icons: HashMap<String, Arc<RenderImage>>,
    background_image_urls: HashMap<Uuid, String>,
    server_icon_urls: HashMap<Uuid, String>,
    background_loading: HashSet<Uuid>,
    icons_loading: HashSet<Uuid>,
    optional_mod_icons_loading: HashSet<String>,
    /// Картинки, которых нет: 404, оборванная ссылка, отказ провайдера.
    ///
    /// Без этого списка неудача ничем не отличалась от «ещё не пробовали»:
    /// запрос уходил заново на каждом кадре, а завершение каждой попытки
    /// дёргало перерисовку — то есть следующий кадр, то есть следующую попытку.
    /// Десяток битых иконок в выдаче каталога так укладывал весь интерфейс.
    optional_mod_icons_failed: HashSet<String>,
    /// То же для фонов и значков сборок, по адресу картинки.
    image_failed: HashSet<String>,

    pub update_available: Option<LauncherVersion>,
    pub updating: bool,
    /// Стопка плашек, старые сверху. Одна на всё окно теряла предыдущую:
    /// синхронизация умеет сообщить о трёх вещах подряд, и видно было третью.
    pub toasts: Vec<Toast>,
    next_toast_id: u64,
    pub config: UiConfig,
    pub server_settings: HashMap<Uuid, ClientSettingsState>,
    pub server_recommendations: HashMap<Uuid, ClientSettingsState>,
    pub console_window: Option<gpui::WindowHandle<ConsoleWindow>>,
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
    /// Показывать в списке версий и те, что сборке не подходят.
    ///
    /// По умолчанию выключено: у популярного мода полсотни версий, из них
    /// подходит одна-две, и искать их глазами среди строк «не выпущена под эту
    /// сборку» — не выбор, а поиск. Но список не прячется совсем: увидеть, что
    /// мод вообще существует под другие версии, бывает важно.
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
    /// Сборка, для которой каталог уже спрашивали, и висит ли запрос сейчас.
    ///
    /// Без этих двух полей условие «список пуст — спроси» срабатывало на
    /// каждом кадре: пустая выдача или ещё не пришедший ответ давали шестьдесят
    /// запросов в секунду, и лагал от этого не только лаунчер, но и мастер, —
    /// он на каждый из них ходил в Modrinth.
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
    /// Какие списки аккаунта уже спрашивали.
    ///
    /// Проверка «список пуст — спроси» стоит в рендере, то есть срабатывает на
    /// каждом кадре. У игрока без наказаний, без обращений или до ответа
    /// мастера это давало запрос на кадр — шестьдесят в секунду, каждый со
    /// своей перерисовкой по ответу. Отсюда и «лагают все списки».
    pub account_requested: HashSet<&'static str>,
    pub punishments: Vec<bridge::PunishmentView>,
    pub rules: Vec<bridge::RuleView>,
    pub rules_query: String,
    pub rules_focus: Option<gpui::FocusHandle>,
    pub tickets: Vec<bridge::TicketView>,
    /// The ticket that is open, with its thread.
    pub ticket_open: Option<(Uuid, String, String, Vec<bridge::TicketMessageView>)>,
    pub dm_threads: Vec<bridge::DmThreadView>,
    /// Список переписок уже спрашивали. Проверка по пустому списку не годится:
    /// она стоит в рендере и у аккаунта без переписок давала запрос на кадр.
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
pub use crate::console_window::ConsoleWindow;
use crate::console_window::MAX_LOG_LINES;

pub struct GlobalLauncherUI(pub Entity<LauncherUI>);
impl gpui::Global for GlobalLauncherUI {}

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
            perf: Default::default(),
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
            installed_keys: HashMap::new(),
            allow_mod_suggestions: HashMap::new(),
            allow_personal_content: HashMap::new(),
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
            optional_mod_icons_failed: HashSet::new(),
            image_failed: HashSet::new(),
            update_available: None,
            impersonate_prompt: None,
            log_request_prompt: None,
            log_request_preview_open: false,
            remote_action_prompt: None,
            impersonating_as: None,
            updating: false,
            toasts: Vec::new(),
            next_toast_id: 0,
            config: UiConfig::default(),
            server_settings: HashMap::new(),
            server_recommendations: HashMap::new(),
            console_window: None,

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

        self.background_images.remove(&server_id);
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
                        // Старую текстуру возвращаем GPUI: атлас держит каждый
                        // `RenderImage` по id и сам ничего не вытесняет, так что
                        // смена фона иначе оставляла бы за собой мегабайты.
                        if let Some(stale) = state.background_images.insert(server_id, image) {
                            if Arc::strong_count(&stale) == 1 {
                                cx.drop_image(stale, None);
                            }
                        }
                    }
                    Err(err) => {
                        // Один раз на адрес. Раньше неудача не запоминалась, и
                        // следующий кадр качал снова — вместе с новым тостом
                        // об ошибке на каждую попытку.
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
        self.server_icons.remove(&server_id);
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

    /// Сколько картинок так и не загрузилось. Оверлею — чтобы отличить
    /// «иконок нет» от «иконки не приходят».
    pub fn failed_image_count(&self) -> usize {
        self.optional_mod_icons_failed.len() + self.image_failed.len()
    }

    pub fn ensure_optional_mod_icon_loaded(&mut self, url: Option<String>, cx: &mut Context<Self>) {
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
            let result = crate::image_loader::load_render_image_capped(url.clone(), 128).await;
            let _ = this.update(cx, |state, cx| {
                state.optional_mod_icons_loading.remove(&url);
                match result {
                    Ok(image) => {
                        state.optional_mod_icons.insert(url, image);
                        // Перерисовка только когда есть что показать: иначе
                        // неудача сама вызывает кадр, который её повторит.
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
            let name = format!("Skin {}", num);
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

    pub fn on_message(&mut self, msg: MessageToFrontend, cx: &mut Context<Self>) {
        match msg {
            MessageToFrontend::LoginSuccess { user } => {
                // Счётчик у колокольчика обязан быть верным до того, как панель
                // откроют: непрочитанное, пришедшее офлайн, иначе не видно
                // вовсе.
                self.backend.send(MessageToBackend::RequestNotifications {
                    offset: 0,
                    unread_only: false,
                });
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
                    // `r` is a translation key from the master, not text.
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
                console,
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
                    console,
                };
                self.server_settings = server_settings.into_iter().collect();
                self.load_preset_renders(cx);
                if self.user.is_none() {
                    self.startup_checking = false;
                }
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
                if stage.is_download() {
                    s.stages.insert(stage, (done, total));
                    // Several stages run at once, so no single one of them gets
                    // to be the heading.
                    s.stage = "Downloading...".into();
                } else {
                    // Checking files opens a new pass; the bars from the last
                    // run don't belong to it.
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
                // Not playable yet: the files get checked and the JVM started,
                // which for a big build takes seconds. The button stays busy
                // until GameStarted or SyncFailed — a second click here started
                // a second game in the same folder.
                let s = self.sync.entry(server_id).or_default();
                s.stage = "Launching...".into();
            }
            MessageToFrontend::LaunchCancelled { server_id } => {
                let s = self.sync.entry(server_id).or_default();
                s.syncing = false;
                s.failed = None;
            }
            MessageToFrontend::LiveSynced {
                server_id,
                updated,
                locked,
            } => {
                // New packs arrived while the game is running. Nothing shows
                // until the client reloads its resources, and that's the
                // player's call: mid-fight it isn't welcome.
                let s = self.sync.entry(server_id).or_default();
                s.stage = if locked.is_empty() {
                    format!("{} pack(s) updated. Press F3+T to apply", updated.len())
                } else {
                    format!(
                        "{} pack(s) updated, {} more will land on next launch",
                        updated.len(),
                        locked.len()
                    )
                };
            }
            MessageToFrontend::SyncFailed { server_id, reason } => {
                let s = self.sync.entry(server_id).or_default();
                s.syncing = false;
                s.failed = Some(reason.clone());
                self.notify_toast(reason, NotifLevel::Error, cx);
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
                    self.notify_toast(i18n::t("error-game-exited"), NotifLevel::Warning, cx);
                }
            }
            MessageToFrontend::GameLog { server_id, lines } => {
                let logs = self.logs.entry(server_id).or_default();
                logs.extend(lines.iter().cloned());
                if logs.len() > MAX_LOG_LINES {
                    let drain = logs.len() - MAX_LOG_LINES;
                    logs.drain(0..drain);
                }
                if let Some(handle) = &self.console_window {
                    let _ = handle.update(cx, |view, _, cx| {
                        if view.server_id == server_id {
                            view.append(lines);
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
                // Тему и статус несёт список: ручка сообщений отдаёт только их
                // самих, и подставить сюда пустые строки значило бы стереть
                // заголовок открытого обращения.
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

    /// Показать плашку и снять её по таймеру.
    ///
    /// Таймер спит в фоне и будит окно один раз — на снятие. Считать оставшееся
    /// время в самом рендере значило бы держать перерисовку все эти секунды,
    /// то есть жечь кадры ради затухающей надписи.
    pub fn notify_toast(&mut self, text: String, level: NotifLevel, cx: &mut Context<Self>) {
        // Тот же текст, что уже висит, второй плашкой не становится: две
        // одинаковые строки рядом выглядят как сбой, а не как два события.
        // Продлеваем ту, что есть, — таймер у неё уже свой.
        if let Some(existing) = self.toasts.iter().find(|t| t.text == text) {
            let id = existing.id;
            let lifetime = existing.lifetime();
            let executor = cx.background_executor().clone();
            cx.spawn(async move |this, cx| {
                executor.timer(lifetime).await;
                let _ = this.update(cx, |state, cx| {
                    state.dismiss_toast(id);
                    cx.notify();
                });
            })
            .detach();
            return;
        }

        let id = self.next_toast_id;
        self.next_toast_id += 1;
        let toast = Toast { id, text, level };
        let lifetime = toast.lifetime();

        self.toasts.push(toast);
        // Больше четырёх на экране — это уже не сообщения, а стена; самое
        // старое уходит раньше срока.
        if self.toasts.len() > 4 {
            self.toasts.remove(0);
        }

        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            executor.timer(lifetime).await;
            let _ = this.update(cx, |state, cx| {
                state.dismiss_toast(id);
                cx.notify();
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
        let server_name = self
            .server(&server_id)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        let settings = self.config.console;
        let backend = self.backend.clone();
        let handle = cx.open_window(
            gpui::WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                window_min_size: Some(gpui::size(
                    px(CONSOLE_WINDOW_MIN_SIZE.0),
                    px(CONSOLE_WINDOW_MIN_SIZE.1),
                )),
                // Same chrome as the launcher: transparent titlebar, traffic
                // lights parked off-screen, the bar drawn by `console_chrome`.
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some(gpui::SharedString::new_static("Noro Game Console")),
                    appears_transparent: true,
                    traffic_light_position: Some(gpui::point(px(-120.), px(-120.))),
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
                    ConsoleWindow::new(server_id, server_name, logs, settings, backend)
                })
            },
        );

        if let Ok(h) = handle {
            self.console_window = Some(h);
        }
    }
}
