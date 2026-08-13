//! Обработка сообщений: MessageToBackend (от frontend) и ServerWsMsg (от мастера).

use std::path::PathBuf;

use crate::auth::{discord_oauth, token_store};
use crate::backend::{spawn_sync_and_launch, BackendState, InternalEvent};
use bridge::{
    ClientSettingsState, LoginErrorKind, MessageToBackend, MessageToFrontend, OptionalModInfo,
};
use schema::{ClientWsMsg, ServerWsMsg};
use uuid::Uuid;

impl BackendState {
    /// Запрос манифеста с учётом выбранной игроком версии.
    ///
    /// Без выбора уходит `None`, и мастер отдаёт текущую опубликованную —
    /// поведение по умолчанию не меняется.
    fn request_manifest_msg(&self, server_id: Uuid) -> ClientWsMsg {
        ClientWsMsg::RequestBuildManifest {
            server_id,
            build_id: self
                .ctx
                .config
                .get()
                .selected_build
                .get(&server_id)
                .copied(),
        }
    }

    /// Команда от frontend.
    pub async fn handle_to_backend(&mut self, msg: MessageToBackend) {
        match msg {
            MessageToBackend::StartDiscordLogin { modal_action } => {
                let master_url = self.ctx.config.get().master_url;
                let internal = self.ctx.internal.clone();
                let modal = modal_action.clone();
                modal.set_stage("Waiting for Discord sign in...");
                tokio::spawn(async move {
                    let cancelled = {
                        let m = modal.clone();
                        move || m.is_cancelled()
                    };
                    match discord_oauth::login(&master_url, cancelled).await {
                        Ok(res) => {
                            let _ = internal.send(InternalEvent::LoginCompleted {
                                auth: res.auth,
                                user: res.user,
                            });
                        }
                        Err(e) => {
                            let kind = if e.to_string().contains("cancel") {
                                LoginErrorKind::Cancelled
                            } else {
                                LoginErrorKind::Network(e.to_string())
                            };
                            let _ = internal.send(InternalEvent::LoginFailed { kind });
                        }
                    }
                });
            }

            MessageToBackend::StartOAuth2Login { modal_action } => {
                let master_url = self.ctx.config.get().master_url;
                let internal = self.ctx.internal.clone();
                let modal = modal_action.clone();
                modal.set_stage("Waiting for Web OAuth2 sign in...");
                tokio::spawn(async move {
                    let cancelled = {
                        let m = modal.clone();
                        move || m.is_cancelled()
                    };
                    match discord_oauth::login_oauth2(&master_url, cancelled).await {
                        Ok(res) => {
                            let _ = internal.send(InternalEvent::LoginCompleted {
                                auth: res.auth,
                                user: res.user,
                            });
                        }
                        Err(e) => {
                            let kind = if e.to_string().contains("cancel") {
                                LoginErrorKind::Cancelled
                            } else {
                                LoginErrorKind::Network(e.to_string())
                            };
                            let _ = internal.send(InternalEvent::LoginFailed { kind });
                        }
                    }
                });
            }

            MessageToBackend::StartKeyLogin { key, modal_action } => {
                let master = self.ctx.config.get().master_url;
                let http = self.ctx.http.clone();
                let internal = self.ctx.internal.clone();
                modal_action.set_stage("Checking authorization key...");

                tokio::spawn(async move {
                    let base = master.trim_end_matches('/');
                    let res = http
                        .get(format!("{base}/api/me"))
                        .header("Authorization", format!("Bearer {key}"))
                        .send()
                        .await;

                    match res {
                        Ok(r) if r.status().is_success() => {
                            if let Ok(profile) = r.json::<schema::UserProfile>().await {
                                modal_action.finish();
                                let auth = token_store::StoredAuth {
                                    access_token: key,
                                    refresh_token: String::new(),
                                };
                                let _ = internal.send(InternalEvent::LoginCompleted {
                                    auth,
                                    user: profile,
                                });
                            }
                        }
                        Ok(r) => {
                            let txt = r.text().await.unwrap_or_default();
                            modal_action.fail(txt.clone());
                            let _ = internal.send(InternalEvent::LoginFailed {
                                kind: bridge::LoginErrorKind::Rejected(if txt.is_empty() {
                                    "Invalid access key".into()
                                } else {
                                    txt
                                }),
                            });
                        }
                        Err(e) => {
                            modal_action.fail(e.to_string());
                            let _ = internal.send(InternalEvent::LoginFailed {
                                kind: bridge::LoginErrorKind::Network(e.to_string()),
                            });
                        }
                    }
                });
            }

            MessageToBackend::StartBiometricLogin { modal_action } => {
                let master = self.ctx.config.get().master_url;
                let http = self.ctx.http.clone();
                let internal = self.ctx.internal.clone();
                let modal = modal_action.clone();
                modal.set_stage("Waiting for biometric authentication...");

                tokio::spawn(async move {
                    // 1. Попытка нативного Touch ID / Windows Hello из системного Keyring
                    if let Ok(true) = crate::auth::biometrics::authenticate_biometrics(
                        "Авторизация в Noro Launcher",
                    ) {
                        if let Some(stored) = token_store::load() {
                            let key = stored.access_token.clone();
                            let base = master.trim_end_matches('/');
                            let res = http
                                .get(format!("{base}/api/me"))
                                .header("Authorization", format!("Bearer {key}"))
                                .send()
                                .await;

                            if let Ok(r) = res {
                                if r.status().is_success() {
                                    if let Ok(profile) = r.json::<schema::UserProfile>().await {
                                        modal.finish();
                                        let auth = token_store::StoredAuth {
                                            access_token: key,
                                            refresh_token: stored.refresh_token,
                                        };
                                        let _ = internal.send(InternalEvent::LoginCompleted {
                                            auth,
                                            user: profile,
                                        });
                                        return;
                                    }
                                }
                            }
                        }
                    }

                    // 2. Если токена в Keyring нет — запускаем вход через Web Passkeys / OAuth в браузере
                    modal.set_stage("Waiting for Passkey sign in in browser...");
                    let cancelled = {
                        let m = modal.clone();
                        move || m.is_cancelled()
                    };
                    match crate::auth::discord_oauth::login_passkey(&master, cancelled).await {
                        Ok(res) => {
                            modal.finish();
                            let _ = internal.send(InternalEvent::LoginCompleted {
                                auth: res.auth,
                                user: res.user,
                            });
                        }
                        Err(e) => {
                            let kind = if e.to_string().contains("cancel") {
                                bridge::LoginErrorKind::Cancelled
                            } else {
                                bridge::LoginErrorKind::Network(e.to_string())
                            };
                            modal.fail(e.to_string());
                            let _ = internal.send(InternalEvent::LoginFailed { kind });
                        }
                    }
                });
            }

            MessageToBackend::Logout => {
                let _ = token_store::clear();
                self.access_token = None;
                self.user = None;
                self.ctx.ws.set_token(None);
                self.ctx.send(MessageToFrontend::LoggedOut);
            }

            MessageToBackend::RequestServerList => {
                self.ctx.ws.send(ClientWsMsg::RequestServerList);
                // Отдадим кэш сразу, если есть.
                if !self.servers.is_empty() {
                    self.ctx.send(MessageToFrontend::ServerList {
                        servers: self.servers.clone(),
                    });
                }
            }

            MessageToBackend::RequestNews => {
                self.ctx.ws.send(ClientWsMsg::RequestNews);
            }

            MessageToBackend::OpenServer { server_id } => {
                if let Some(srv) = self.servers.iter().find(|s| s.id == server_id) {
                    if self.ctx.running.lock().is_empty() {
                        self.ctx
                            .rpc
                            .update(crate::discord_rpc::DiscordRpcState::Launcher {
                                server_name: Some(srv.name.clone()),
                            });
                    }
                }
                if let Some(manifest) = self.manifests.get(&server_id).cloned() {
                    self.send_server_recommendation(server_id, &manifest);
                    self.send_optional_mods(server_id, &manifest);
                } else {
                    self.ctx.ws.send(self.request_manifest_msg(server_id));
                }
            }

            MessageToBackend::LaunchServer {
                server_id,
                modal_action,
            } => {
                if let Some(srv) = self.servers.iter().find(|s| s.id == server_id) {
                    self.ctx
                        .rpc
                        .update(crate::discord_rpc::DiscordRpcState::GameLoading {
                            server_name: srv.name.clone(),
                        });
                }
                self.launch_server(server_id, modal_action).await;
            }

            MessageToBackend::KillGame { server_id } => {
                if let Some(g) = self.ctx.running.lock().get(&server_id) {
                    let _ = g.kill.send(());
                }
            }

            MessageToBackend::SetOptionalMods { server_id, enabled } => {
                self.ctx.optional.update(|s| {
                    s.enabled.insert(server_id, enabled.clone());
                });
                self.ctx
                    .ws
                    .send(ClientWsMsg::SetOptionalMods { server_id, enabled });
            }

            MessageToBackend::SelectBuild {
                server_id,
                build_id,
            } => {
                self.ctx.config.update(|c| match build_id {
                    Some(id) => {
                        c.selected_build.insert(server_id, id);
                    }
                    // Возврат к текущей версии — это отсутствие записи, а не
                    // запомненный id: иначе выбор «залипнет» на старой сборке,
                    // когда админ выкатит новую.
                    None => {
                        c.selected_build.remove(&server_id);
                    }
                });
                // Манифест перезапрашивается сразу: игрок ждёт, что список
                // файлов и модов обновится под выбранную версию.
                self.ctx.ws.send(self.request_manifest_msg(server_id));
            }

            MessageToBackend::SuggestOptionalMod {
                server_id,
                build_id,
                provider,
                project_id,
                title,
                icon_url,
                description,
            } => {
                let Some(token) = self.access_token.clone() else {
                    self.ctx.send(MessageToFrontend::AddNotification {
                        key: "notif-sign-in-to-suggest".into(),
                        args: std::collections::BTreeMap::new(),
                        level: schema::NotifLevel::Error,
                    });
                    return;
                };
                let ctx = self.ctx.clone();
                let http = self.ctx.http.clone();
                tokio::spawn(async move {
                    let master_url = ctx.config.get().master_url;

                    let res = http
                        .post(format!("{master_url}/api/mod_suggestions"))
                        .bearer_auth(&token)
                        .json(&serde_json::json!({
                            "server_id": server_id,
                            "build_id": build_id,
                            "provider": provider,
                            "project_id": project_id,
                            "title": title,
                            "icon_url": icon_url,
                            "description": description,
                        }))
                        .send()
                        .await;

                    match res {
                        Ok(res) if res.status().is_success() => {
                            ctx.send(MessageToFrontend::AddNotification {
                                key: "Mod request submitted to admin!".into(),
                                args: std::collections::BTreeMap::new(),
                                level: schema::NotifLevel::Info,
                            });
                        }
                        Ok(res) => {
                            ctx.send(MessageToFrontend::AddNotification {
                                key: format!("Failed to submit request ({})", res.status()),
                                args: std::collections::BTreeMap::new(),
                                level: schema::NotifLevel::Error,
                            });
                        }
                        Err(e) => {
                            ctx.send(MessageToFrontend::AddNotification {
                                key: format!("Network error: {e}"),
                                args: std::collections::BTreeMap::new(),
                                level: schema::NotifLevel::Error,
                            });
                        }
                    }
                });
            }

            MessageToBackend::SearchCatalog {
                query,
                provider,
                mc_version,
                loader,
                offset,
            } => {
                let ctx = self.ctx.clone();
                let http = self.ctx.http.clone();
                tokio::spawn(async move {
                    let master_url = ctx.config.get().master_url;
                    let page = crate::catalog_search::search(
                        &http,
                        &master_url,
                        &query,
                        &provider,
                        mc_version.as_deref(),
                        loader.as_deref(),
                        offset,
                    )
                    .await;
                    match page {
                        Ok(page) => ctx.send(MessageToFrontend::CatalogSearchResults {
                            hits: page.hits,
                            total: page.total,
                            offset: page.offset,
                            limit: page.limit,
                        }),
                        Err(e) => {
                            tracing::error!(error = %e, "поиск в каталоге не удался");
                            ctx.send(MessageToFrontend::CatalogFailed {
                                message: e.to_string(),
                            });
                        }
                    }
                });
            }

            MessageToBackend::RequestModProject {
                provider,
                project_id,
            } => {
                let ctx = self.ctx.clone();
                let http = self.ctx.http.clone();
                tokio::spawn(async move {
                    let master_url = ctx.config.get().master_url;
                    let url = format!(
                        "{master_url}/api/admin/catalog/{}/project/{}",
                        urlencoding::encode(&provider),
                        urlencoding::encode(&project_id),
                    );
                    // Поля страницы совпадают с ModProjectInfo по именам, а всё
                    // лишнее из ответа мастера serde просто игнорирует.
                    let loaded = async {
                        http.get(&url)
                            .send()
                            .await?
                            .error_for_status()?
                            .json::<bridge::ModProjectInfo>()
                            .await
                    }
                    .await;
                    match loaded {
                        Ok(project) => ctx.send(MessageToFrontend::ModProjectLoaded { project }),
                        Err(e) => {
                            tracing::error!(error = %e, "страница мода не загрузилась");
                            ctx.send(MessageToFrontend::CatalogFailed {
                                message: e.to_string(),
                            });
                        }
                    }
                });
            }

            MessageToBackend::SetMemory { min_mb, max_mb } => {
                self.ctx.config.update(|c| {
                    c.memory_min_mb = min_mb;
                    c.memory_max_mb = max_mb.max(min_mb);
                });
            }

            MessageToBackend::SetJvmFlags { flags } => {
                self.ctx.config.update(|c| c.jvm_flags = flags);
            }

            MessageToBackend::SetShowConsoleOnLaunch { enabled } => {
                self.ctx
                    .config
                    .update(|c| c.show_console_on_launch = enabled);
            }

            MessageToBackend::SetCrashReports { enabled } => {
                // Применится со следующего запуска: Sentry поднимается до GPUI,
                // а снять уже установленный хук паники на ходу нельзя.
                self.ctx.config.update(|c| c.crash_reports = enabled);
                self.send_config_state();
            }

            MessageToBackend::SetServerMemory {
                server_id,
                min_mb,
                max_mb,
            } => {
                self.ctx
                    .config
                    .update(|c| c.set_server_memory(server_id, min_mb, max_mb));
            }

            MessageToBackend::SetServerJvmFlags { server_id, flags } => {
                self.ctx
                    .config
                    .update(|c| c.set_server_jvm_flags(server_id, flags));
            }

            MessageToBackend::SetServerShowConsoleOnLaunch { server_id, enabled } => {
                self.ctx
                    .config
                    .update(|c| c.set_server_console(server_id, enabled));
            }

            MessageToBackend::ResetServerClientSettings { server_id } => {
                self.ctx
                    .config
                    .update(|c| c.reset_server_settings(&server_id));
            }

            MessageToBackend::OpenServerClientFolder { server_id } => {
                let client_path: PathBuf = self.ctx.dirs.instance(&server_id);
                let _ = open::that(client_path);
            }

            MessageToBackend::SetLocale { code } => {
                self.ctx.config.update(|c| c.locale = code.clone());
                crate::translations::refresh(&self.ctx, code);
            }

            MessageToBackend::InstallUpdate {
                version,
                modal_action,
            } => {
                let ctx = self.ctx.clone();
                let modal = modal_action.clone();
                modal.set_stage("Downloading update...");
                tokio::spawn(async move {
                    let result = crate::updater::install_update(
                        &ctx.http,
                        &ctx.dirs,
                        &version,
                        |done, total| {
                            modal.set_progress(done, total);
                        },
                    )
                    .await;
                    match result {
                        Ok(exe) => {
                            modal.finish();
                            let _ = ctx.internal.send(InternalEvent::RestartInto(exe));
                        }
                        Err(e) => {
                            modal.fail(e.to_string());
                            ctx.send(MessageToFrontend::AddNotification {
                                key: "notif-update-failed".into(),
                                args: [("reason".to_string(), e.to_string())].into(),
                                level: schema::NotifLevel::Error,
                            });
                        }
                    }
                });
            }

            MessageToBackend::UploadSkin { bytes } => {
                if let Some(token) = &self.access_token {
                    let master = self.ctx.config.get().master_url.clone();
                    let http = self.ctx.http.clone();
                    let t = token.clone();
                    let b = bytes.clone();
                    let internal = self.ctx.internal.clone();
                    let ctx2 = self.ctx.clone();
                    tokio::spawn(async move {
                        match upload_skin_to_master(&http, &master, &t, b).await {
                            Ok(profile) => {
                                let _ =
                                    internal.send(InternalEvent::ProfileUpdated { user: profile });
                            }
                            Err(e) => {
                                ctx2.send(MessageToFrontend::AddNotification {
                                    key: "notif-skin-upload-failed".into(),
                                    args: [("reason".to_string(), e.to_string())].into(),
                                    level: schema::NotifLevel::Error,
                                });
                                ctx2.send(MessageToFrontend::SkinUploadFailed);
                            }
                        }
                    });
                } else {
                    self.ctx.send(MessageToFrontend::AddNotification {
                        key: "notif-sign-in-to-upload".into(),
                        args: Default::default(),
                        level: schema::NotifLevel::Error,
                    });
                }
            }

            MessageToBackend::RequestCapesList => {
                if let Some(token) = &self.access_token {
                    let master = self.ctx.config.get().master_url.clone();
                    let http = self.ctx.http.clone();
                    let t = token.clone();
                    let ctx = self.ctx.clone();
                    tokio::spawn(async move {
                        if let Ok(capes) = fetch_capes_from_master(&http, &master, &t).await {
                            ctx.send(MessageToFrontend::CapesList { capes });
                        }
                    });
                }
            }

            MessageToBackend::RequestSkinPresetsList => {
                if let Some(token) = &self.access_token {
                    let master = self.ctx.config.get().master_url.clone();
                    let http = self.ctx.http.clone();
                    let t = token.clone();
                    let ctx = self.ctx.clone();
                    tokio::spawn(async move {
                        if let Ok(presets) =
                            fetch_skin_presets_from_master(&http, &master, &t).await
                        {
                            ctx.send(MessageToFrontend::SkinPresetsList { presets });
                        }
                    });
                }
            }

            MessageToBackend::SelectCape { cape_id } => {
                if let Some(token) = &self.access_token {
                    let master = self.ctx.config.get().master_url.clone();
                    let http = self.ctx.http.clone();
                    let t = token.clone();
                    let internal = self.ctx.internal.clone();
                    let ctx = self.ctx.clone();
                    tokio::spawn(async move {
                        match select_cape_on_master(&http, &master, &t, cape_id).await {
                            Ok(profile) => {
                                let _ = internal.send(InternalEvent::ProfileUpdated {
                                    user: profile.clone(),
                                });
                                ctx.send(MessageToFrontend::PermissionsUpdated { user: profile });
                            }
                            Err(e) => {
                                ctx.send(MessageToFrontend::AddNotification {
                                    key: "notif-cape-update-failed".into(),
                                    args: [("reason".to_string(), e.to_string())].into(),
                                    level: schema::NotifLevel::Error,
                                });
                            }
                        }
                    });
                }
            }

            MessageToBackend::FocusWindow => {
                self.ctx.send(MessageToFrontend::OpenOrFocusMainWindow);
            }

            MessageToBackend::Quit => {
                // Убить запущенные игры.
                let running: Vec<_> = self.ctx.running.lock().keys().copied().collect();
                for id in running {
                    if let Some(g) = self.ctx.running.lock().get(&id) {
                        let _ = g.kill.send(());
                    }
                }
            }
        }
    }

    /// Запуск сервера: запросить манифест (или взять кэш), затем sync+launch.
    async fn launch_server(&mut self, server_id: Uuid, modal: bridge::ModalAction) {
        if self.login_info().is_none() {
            modal.fail("Sign in required");
            self.ctx.send(MessageToFrontend::SyncFailed {
                server_id,
                reason: "not signed in".into(),
            });
            return;
        }
        if self.ctx.running.lock().contains_key(&server_id) {
            self.ctx.send(MessageToFrontend::AddNotification {
                key: "notif-already-running".into(),
                args: Default::default(),
                level: schema::NotifLevel::Warning,
            });
            return;
        }

        modal.set_stage("Fetching build manifest...");
        self.pending_launch.insert(server_id, modal);

        if let Some(manifest) = self.manifests.get(&server_id).cloned() {
            self.begin_launch(server_id, manifest);
        } else {
            self.ctx.ws.send(self.request_manifest_msg(server_id));
        }
    }

    /// Начать sync+launch для уже доступного манифеста.
    fn begin_launch(&mut self, server_id: Uuid, manifest: schema::BuildManifest) {
        let Some(modal) = self.pending_launch.remove(&server_id) else {
            return; // не мы инициировали
        };
        let (Some(login), Some(user)) = (self.login_info(), self.user.clone()) else {
            return;
        };
        let enabled = self.ctx.optional.get().for_server(&server_id);
        let connect = self.server_connect(&server_id);
        spawn_sync_and_launch(
            self.ctx.clone(),
            server_id,
            manifest,
            user,
            login,
            connect,
            enabled,
            self.servers.iter().find(|s| s.id == server_id).cloned(),
            modal,
        );
    }

    /// Отправить текущую конфигурацию во frontend.
    pub fn send_config_state(&self) {
        let c = self.ctx.config.get();
        let server_settings = c
            .server_settings
            .iter()
            .map(|(id, settings)| {
                (
                    *id,
                    ClientSettingsState {
                        memory_min_mb: settings.memory_min_mb,
                        memory_max_mb: settings.memory_max_mb,
                        jvm_flags: settings.jvm_flags.clone(),
                        show_console_on_launch: settings.show_console_on_launch,
                    },
                )
            })
            .collect();
        self.ctx.send(MessageToFrontend::ConfigState {
            memory_min_mb: c.memory_min_mb,
            memory_max_mb: c.memory_max_mb,
            jvm_flags: c.jvm_flags,
            locale: c.locale.clone(),
            show_console_on_launch: c.show_console_on_launch,
            crash_reports: c.crash_reports,
            crash_reports_available: crate::telemetry::is_available(),
            master_url: c.master_url,
            server_settings,
        });
    }

    /// Вычислить и отправить опциональные моды сервера (с учётом прав).
    /// Сообщить фронту, что сейчас можно сделать со сборкой.
    fn send_build_state(&self, server_id: uuid::Uuid, manifest: &schema::BuildManifest) {
        let dir = self.ctx.dirs.instance(&server_id);
        self.ctx.send(bridge::MessageToFrontend::BuildStateChanged {
            server_id,
            state: crate::sync::build_state(&dir, manifest),
        });
    }

    fn send_optional_mods(&self, server_id: Uuid, manifest: &schema::BuildManifest) {
        use crate::directories::safe_join;
        let enabled = self.ctx.optional.get().for_server(&server_id);
        let instance_dir = self.ctx.dirs.instance(&server_id);
        let mods = manifest
            .optional_mods
            .iter()
            .filter(|m| m.visible)
            .map(|m| {
                let allowed = self
                    .user
                    .as_ref()
                    .map(|u| u.can_use_optional(&server_id, &m.name, m.limited))
                    .unwrap_or(!m.limited);
                let is_enabled = if enabled.is_empty() {
                    m.enabled_by_default
                } else {
                    enabled.contains(&m.name)
                };
                let icon_url = m.icon_url.clone().or_else(|| {
                    m.files
                        .iter()
                        .filter(|f| f.ends_with(".jar"))
                        .find_map(|f| {
                            let path = safe_join(&instance_dir, f)?;
                            crate::mod_icon::extract_jar_icon(&path)
                        })
                });
                OptionalModInfo {
                    name: m.name.clone(),
                    description: m.description.clone(),
                    category: m.category.clone(),
                    icon_url,
                    author: m.author.clone(),
                    limited: m.limited,
                    allowed,
                    enabled: is_enabled && allowed,
                }
            })
            .collect();
        let installed_files = manifest
            .verified_files
            .iter()
            .map(|f| f.path.clone())
            .collect();
        self.ctx.send(MessageToFrontend::OptionalMods {
            server_id,
            mods,
            allow_suggestions: manifest.allow_optional_mod_suggestions,
            installed_files,
        });
    }

    fn send_server_recommendation(&self, server_id: Uuid, manifest: &schema::BuildManifest) {
        let settings = &manifest.recommended_client_settings;
        self.ctx
            .send(MessageToFrontend::ServerClientRecommendation {
                server_id,
                settings: ClientSettingsState {
                    memory_min_mb: settings.memory_min_mb,
                    memory_max_mb: settings.memory_max_mb,
                    jvm_flags: settings.jvm_flags.clone(),
                    show_console_on_launch: settings.show_console_on_launch,
                },
            });
    }

    /// Сообщение от мастера.
    pub async fn handle_from_master(&mut self, msg: ServerWsMsg) {
        match msg {
            ServerWsMsg::AuthOk { user } => {
                self.user = Some(user.clone());
                self.ctx.send(MessageToFrontend::LoginSuccess { user });
                if let Some(token) = &self.access_token {
                    let master = self.ctx.config.get().master_url.clone();
                    let http = self.ctx.http.clone();
                    let t = token.clone();
                    let ctx = self.ctx.clone();
                    tokio::spawn(async move {
                        if let Ok(presets) =
                            fetch_skin_presets_from_master(&http, &master, &t).await
                        {
                            ctx.send(MessageToFrontend::SkinPresetsList { presets });
                        }
                    });
                }
            }
            ServerWsMsg::AuthFail { reason } => {
                tracing::warn!("auth fail: {reason}");
                // Токен недействителен — выходим.
                let _ = token_store::clear();
                self.access_token = None;
                self.user = None;
                self.ctx.ws.set_token(None);
                self.ctx.send(MessageToFrontend::LoggedOut);
            }
            ServerWsMsg::ServerList { servers } => {
                self.servers = servers.clone();
                self.ctx.send(MessageToFrontend::ServerList { servers });
            }
            ServerWsMsg::News { items } => {
                self.ctx.send(MessageToFrontend::NewsUpdated { items });
            }
            ServerWsMsg::BuildManifest { manifest } => {
                let server_id = manifest.server_id;
                self.manifests.insert(server_id, manifest.clone());
                self.send_server_recommendation(server_id, &manifest);
                // Всегда отдаём опц. моды во frontend (для карточки сервера).
                self.send_optional_mods(server_id, &manifest);
                self.send_build_state(server_id, &manifest);
                if self.pending_launch.contains_key(&server_id) {
                    self.begin_launch(server_id, manifest);
                }
            }
            ServerWsMsg::LauncherUpdate { version } => {
                self.ctx
                    .send(MessageToFrontend::LauncherUpdateAvailable { version });
            }
            ServerWsMsg::Notification { key, args, level } => {
                self.ctx
                    .send(MessageToFrontend::AddNotification { key, args, level });
            }
            ServerWsMsg::ServersChanged => {
                self.ctx.ws.send(ClientWsMsg::RequestServerList);
            }
            ServerWsMsg::TranslationsChanged => {
                let code = self.ctx.config.get().locale;
                crate::translations::refresh(&self.ctx, code);
            }

            ServerWsMsg::NewsChanged => {
                self.ctx.ws.send(ClientWsMsg::RequestNews);
            }
            ServerWsMsg::BuildsChanged { server_id } => {
                let had_manifest = self.manifests.remove(&server_id).is_some();
                self.ctx.ws.send(ClientWsMsg::RequestServerList);
                if self.user.is_some()
                    && (had_manifest || self.pending_launch.contains_key(&server_id))
                {
                    self.ctx.ws.send(self.request_manifest_msg(server_id));
                }
            }
            ServerWsMsg::PermissionsUpdated { user } => {
                self.user = Some(user.clone());
                self.ctx
                    .send(MessageToFrontend::PermissionsUpdated { user });
                self.ctx.ws.send(ClientWsMsg::RequestServerList);
                for (server_id, manifest) in &self.manifests {
                    self.send_optional_mods(*server_id, manifest);
                }
            }
            ServerWsMsg::Pong => {}
        }
    }
}

/// Multipart upload of skin bytes to master using the launcher access token (Bearer).
/// Returns the fresh UserProfile from /api/me/skin (same as cabinet).
async fn upload_skin_to_master(
    http: &reqwest::Client,
    master: &str,
    token: &str,
    bytes: Vec<u8>,
) -> Result<schema::UserProfile, String> {
    let base = master.trim_end_matches('/');
    let url = format!("{}/api/me/skin", base);
    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name("skin.png")
        .mime_str("image/png")
        .map_err(|e| e.to_string())?;
    let form = reqwest::multipart::Form::new().part("skin", part);
    let res = http
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let txt = res.text().await.unwrap_or_default();
        return Err(format!("HTTP {} {}", status, txt));
    }
    res.json::<schema::UserProfile>()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_capes_from_master(
    http: &reqwest::Client,
    master: &str,
    token: &str,
) -> Result<Vec<schema::CapeRow>, String> {
    let base = master.trim_end_matches('/');
    let url = format!("{}/api/capes", base);
    let res = http
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let txt = res.text().await.unwrap_or_default();
        return Err(format!("HTTP {} {}", status, txt));
    }
    res.json::<Vec<schema::CapeRow>>()
        .await
        .map_err(|e| e.to_string())
}

async fn select_cape_on_master(
    http: &reqwest::Client,
    master: &str,
    token: &str,
    cape_id: Option<uuid::Uuid>,
) -> Result<schema::UserProfile, String> {
    let base = master.trim_end_matches('/');
    let url = format!("{}/api/me/cape", base);
    let req = schema::SelectCapeReq { cape_id };
    let res = http
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&req)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let txt = res.text().await.unwrap_or_default();
        return Err(format!("HTTP {} {}", status, txt));
    }
    res.json::<schema::UserProfile>()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_skin_presets_from_master(
    http: &reqwest::Client,
    master: &str,
    token: &str,
) -> Result<Vec<bridge::ServerSkinPresetItem>, String> {
    let base = master.trim_end_matches('/');
    let url = format!("{}/api/me/skin-presets", base);
    let res = http
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        let status = res.status();
        let txt = res.text().await.unwrap_or_default();
        return Err(format!("HTTP {} {}", status, txt));
    }
    res.json::<Vec<bridge::ServerSkinPresetItem>>()
        .await
        .map_err(|e| e.to_string())
}
