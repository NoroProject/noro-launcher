//! Message handling: `MessageToBackend` from the frontend, `ServerWsMsg` from
//! the master.

use std::path::PathBuf;

use crate::auth::{token_store, web_login};
use crate::backend::{spawn_sync_and_launch, BackendState, InternalEvent};
use bridge::{
    ClientSettingsState, LoginErrorKind, MessageToBackend, MessageToFrontend, OptionalModInfo,
};
use schema::{ClientWsMsg, ServerWsMsg};
use uuid::Uuid;

impl BackendState {
    /// `build_id` stays `None` while the player has not pinned a version, and
    /// the master answers with whatever is published right now.
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

    pub async fn handle_to_backend(&mut self, msg: MessageToBackend) {
        match msg {
            MessageToBackend::StartWebLogin { modal_action } => {
                let master_url = self.ctx.config.get().master_url;
                let internal = self.ctx.internal.clone();
                let modal = modal_action.clone();
                modal.set_stage("Waiting for sign in on the website...");
                tokio::spawn(async move {
                    let cancelled = {
                        let m = modal.clone();
                        move || m.is_cancelled()
                    };
                    match web_login::login(&master_url, cancelled).await {
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
                    if let Ok(true) =
                        crate::auth::biometrics::authenticate_biometrics("Sign in to Noro Launcher")
                    {
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

                    // Nothing usable in the keyring, so fall back to the site.
                    modal.set_stage("Waiting for sign in on the website...");
                    let cancelled = {
                        let m = modal.clone();
                        move || m.is_cancelled()
                    };
                    match web_login::login(&master, cancelled).await {
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
                self.ctx.set_profile(None);
                self.ctx.ws.set_token(None);
                self.ctx.send(MessageToFrontend::LoggedOut);
            }

            MessageToBackend::RequestServerList => {
                self.ctx.ws.send(ClientWsMsg::RequestServerList);
                // Answer from the cache while the request is in flight.
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
                    // Going back to the current version means no entry at all.
                    // Storing the id instead would pin the player to that build
                    // once the admin publishes a newer one.
                    None => {
                        c.selected_build.remove(&server_id);
                    }
                });
                // Re-request right away: the file and mod lists have to follow
                // the version that was just picked.
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
                        .post(format!("{master_url}/api/mod-suggestions"))
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
                            tracing::error!(error = %e, "catalog search failed");
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
                    // The page's fields line up with `ModProjectInfo` by name,
                    // and serde drops whatever else the master sends.
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
                            tracing::error!(error = %e, "mod page failed to load");
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

            MessageToBackend::SetFullscreen { enabled } => {
                self.ctx.config.update(|c| c.fullscreen = enabled);
            }

            MessageToBackend::SetCrashReports { enabled } => {
                // Takes effect on the next start: Sentry comes up before GPUI,
                // and an installed panic hook can't be taken back off.
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

            MessageToBackend::SetServerFullscreen { server_id, enabled } => {
                self.ctx
                    .config
                    .update(|c| c.set_server_fullscreen(server_id, enabled));
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

            MessageToBackend::RemoteActionAnswer {
                action,
                server_id,
                accepted,
            } => {
                if accepted {
                    self.perform_remote_action(action, server_id);
                }
            }

            MessageToBackend::LogRequestAnswer {
                request_id,
                accepted,
            } => {
                self.answer_log_request(request_id, accepted);
            }

            MessageToBackend::ImpersonateAnswer { grant_id, accepted } => {
                self.answer_impersonate(grant_id, accepted);
            }

            MessageToBackend::ImpersonateExit => {
                self.exit_impersonation();
            }

            MessageToBackend::SendSupportBundle { server_id } => {
                self.send_support_bundle(server_id);
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

            MessageToBackend::SetSkinModel { slim } => {
                if let Some(token) = &self.access_token {
                    let master = self.ctx.config.get().master_url.clone();
                    let http = self.ctx.http.clone();
                    let t = token.clone();
                    let internal = self.ctx.internal.clone();
                    let ctx2 = self.ctx.clone();
                    tokio::spawn(async move {
                        match set_skin_model_on_master(&http, &master, &t, slim).await {
                            Ok(profile) => {
                                let _ =
                                    internal.send(InternalEvent::ProfileUpdated { user: profile });
                            }
                            Err(e) => {
                                ctx2.send(MessageToFrontend::AddNotification {
                                    key: "notif-skin-model-failed".into(),
                                    args: [("reason".to_string(), e.to_string())].into(),
                                    level: schema::NotifLevel::Error,
                                });
                            }
                        }
                    });
                }
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
                let running: Vec<_> = self.ctx.running.lock().keys().copied().collect();
                for id in running {
                    if let Some(g) = self.ctx.running.lock().get(&id) {
                        let _ = g.kill.send(());
                    }
                }
            }
        }
    }

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

    /// Sync and launch for a manifest that is already in hand.
    fn begin_launch(&mut self, server_id: Uuid, manifest: schema::BuildManifest) {
        let Some(modal) = self.pending_launch.remove(&server_id) else {
            return; // nobody asked for a launch
        };
        let (Some(login), Some(user)) = (self.login_info(), self.user.clone()) else {
            return;
        };
        let enabled = self.ctx.optional.get().for_server(&server_id);
        let connect = self.server_connect(&server_id);
        spawn_sync_and_launch(crate::backend::Launch {
            ctx: self.ctx.clone(),
            server_id,
            manifest,
            user,
            login,
            connect,
            enabled_optional: enabled,
            server: self.servers.iter().find(|s| s.id == server_id).cloned(),
            modal,
        });
    }

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
                        fullscreen: settings.fullscreen,
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
            fullscreen: c.fullscreen,
            crash_reports: c.crash_reports,
            crash_reports_available: crate::telemetry::is_available(),
            master_url: c.master_url,
            server_settings,
        });
    }

    /// Update packs and shaders without leaving the game.
    ///
    /// Silent when there is nothing to update: a "synced" toast for every
    /// little thing teaches the player to ignore it.
    fn live_sync(&self, server_id: uuid::Uuid, manifest: schema::BuildManifest) {
        let dir = self.ctx.dirs.instance(&server_id);
        let client = self.ctx.http.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match crate::sync::live::apply(&client, &dir, &manifest).await {
                Ok(done) if done.nothing() => {}
                Ok(done) => {
                    // The files on disk changed, but the game still holds the
                    // old ones in memory. Nothing outside the process can make
                    // it reload resources, so the mod is asked to.
                    ctx.mod_link.send(mod_link::ToMod::ReloadResources {
                        packs: done.updated.clone(),
                    });
                    ctx.send(MessageToFrontend::LiveSynced {
                        server_id,
                        updated: done.updated,
                        locked: done.locked,
                    });
                }
                Err(e) => tracing::warn!(error = %e, "live sync failed"),
            }
        });
    }

    /// Tells the frontend what can be done with the build right now.
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
                    conflicts: m.conflicts.clone(),
                    dependencies: m.dependencies.clone(),
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
                    fullscreen: settings.fullscreen,
                },
            });
    }

    pub async fn handle_from_master(&mut self, msg: ServerWsMsg) {
        match msg {
            ServerWsMsg::AuthOk { user } => {
                self.user = Some(user.clone());
                self.ctx.set_profile(Some(user.clone()));
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
                let _ = token_store::clear();
                self.access_token = None;
                self.user = None;
                self.ctx.set_profile(None);
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
                self.send_optional_mods(server_id, &manifest);
                self.send_build_state(server_id, &manifest);
                if self.pending_launch.contains_key(&server_id) {
                    self.begin_launch(server_id, manifest);
                } else {
                    // The game may be running right now. A full sync would
                    // delete files the JVM is holding, but packs and shaders
                    // are read on demand and can be swapped under it.
                    self.live_sync(server_id, manifest);
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
                // The manifest matters even for a build nobody opened this
                // session: a directory on disk means the player uses it, and
                // live sync should reach them without waiting for them to visit
                // the server page.
                let installed = self.ctx.dirs.instance(&server_id).exists();
                if self.user.is_some()
                    && (had_manifest || installed || self.pending_launch.contains_key(&server_id))
                {
                    self.ctx.ws.send(self.request_manifest_msg(server_id));
                }
            }
            ServerWsMsg::PermissionsUpdated { user } => {
                self.user = Some(user.clone());
                self.ctx.set_profile(Some(user.clone()));
                self.ctx
                    .send(MessageToFrontend::PermissionsUpdated { user });
                self.ctx.ws.send(ClientWsMsg::RequestServerList);
                for (server_id, manifest) in &self.manifests {
                    self.send_optional_mods(*server_id, manifest);
                }
            }
            ServerWsMsg::RequestDiagnostics => {
                let ctx = self.ctx.clone();
                let master = self.ctx.config.get().master_url;
                tokio::spawn(async move {
                    let report =
                        crate::diagnostics::collect(&ctx.http, &ctx.dirs, &master, None).await;
                    ctx.ws.send(ClientWsMsg::DiagnosticsReport { report });
                });
            }

            ServerWsMsg::RemoteAction {
                action,
                server_id,
                actor_username,
            } => {
                self.run_remote_action(action, server_id, actor_username);
            }

            ServerWsMsg::LogRequest {
                request_id,
                actor_username,
                reason,
                forced,
                server_id,
                ..
            } => {
                self.prepare_log_request(request_id, actor_username, reason, forced, server_id);
            }
            ServerWsMsg::ImpersonateRequest {
                grant_id,
                actor_username,
                target_username,
                reason,
                expires_at,
            } => {
                let expires_in_secs = (expires_at - chrono::Utc::now()).num_seconds().max(0);
                self.ctx.send(MessageToFrontend::ImpersonatePrompt {
                    grant_id,
                    actor_username,
                    target_username,
                    reason,
                    expires_in_secs,
                });
            }
            // With no game running there is no panel to push this to, which is
            // the ordinary situation rather than a failure.
            ServerWsMsg::CaseUpdated { case_id } => {
                self.ctx.mod_link.case_updated(&self.ctx, case_id);
            }
            ServerWsMsg::Pong => {}
            ServerWsMsg::ModuleMessage { .. } => {}
        }
    }

    /// Anything that erases files or interrupts the player asks them first.
    fn run_remote_action(
        &mut self,
        action: schema::RemoteAction,
        server_id: Option<Uuid>,
        actor_username: String,
    ) {
        if action.needs_confirmation() {
            self.ctx.send(MessageToFrontend::RemoteActionPrompt {
                action,
                server_id,
                actor_username,
            });
            return;
        }
        self.perform_remote_action(action, server_id);
    }

    /// The run itself: after confirmation, or straight away when none is needed.
    pub fn perform_remote_action(&mut self, action: schema::RemoteAction, server_id: Option<Uuid>) {
        if action == schema::RemoteAction::KillGame {
            let running: Vec<_> = self.ctx.running.lock().keys().copied().collect();
            let mut killed = 0;
            for id in running {
                if server_id.is_none() || server_id == Some(id) {
                    if let Some(g) = self.ctx.running.lock().get(&id) {
                        let _ = g.kill.send(());
                        killed += 1;
                    }
                }
            }
            self.ctx.send(MessageToFrontend::AddNotification {
                key: "notif-remote-action-done".into(),
                args: [(
                    "detail".to_string(),
                    format!("game process stopped ({killed})"),
                )]
                .into(),
                level: schema::NotifLevel::Info,
            });
            return;
        }

        if action == schema::RemoteAction::RestartLauncher {
            let ctx = self.ctx.clone();
            tokio::spawn(async move {
                ctx.send(MessageToFrontend::AddNotification {
                    key: "notif-remote-action-done".into(),
                    args: [("detail".to_string(), "the launcher is restarting...".into())].into(),
                    level: schema::NotifLevel::Info,
                });
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if let Ok(current_exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(current_exe).spawn();
                }
                std::process::exit(0);
            });
            return;
        }

        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match crate::remote_actions::run(&ctx.dirs, action, server_id).await {
                Ok(outcome) => {
                    tracing::info!(action = action.as_str(), "{}", outcome.message);
                    ctx.send(MessageToFrontend::AddNotification {
                        key: "notif-remote-action-done".into(),
                        args: [("detail".to_string(), outcome.message)].into(),
                        level: schema::NotifLevel::Info,
                    });
                }
                Err(e) => {
                    tracing::warn!(error = %e, action = action.as_str(), "action failed")
                }
            }
        });
    }

    /// Collect the bundle and show the player exactly what would leave their
    /// machine. Forced mode sends it anyway, but still shows the modal.
    fn prepare_log_request(
        &mut self,
        request_id: Uuid,
        actor_username: String,
        reason: String,
        forced: bool,
        target_server_id: Option<Uuid>,
    ) {
        let server_id = target_server_id.or_else(|| self.manifests.keys().copied().next());
        let instance_dir = match server_id {
            Some(ref id) => self.ctx.dirs.instance(id),
            None => self.ctx.dirs.root.clone(),
        };
        let ctx = self.ctx.clone();
        let token = self.access_token.clone();
        let master = self.ctx.config.get().master_url;

        tokio::spawn(async move {
            let bundle = crate::support::collect(&instance_dir, None, &[]).await;
            let files = bundle
                .files
                .iter()
                .map(|f| (f.name.clone(), f.original_bytes))
                .collect();

            ctx.send(MessageToFrontend::LogRequestPrompt {
                request_id,
                actor_username,
                reason,
                forced,
                preview: bundle.preview(),
                files,
            });

            // Forced mode doesn't wait for the answer — the prompt above is a
            // notice, not a question.
            if forced {
                if let Some(token) = token {
                    let _ = crate::support::send_for_request(
                        &ctx.http,
                        &master,
                        &token,
                        &instance_dir,
                        server_id,
                        request_id,
                    )
                    .await;
                }
            }
        });
    }

    fn answer_log_request(&mut self, request_id: Uuid, accepted: bool) {
        self.ctx.ws.send(ClientWsMsg::LogRequestResponse {
            request_id,
            accepted,
        });
        if !accepted {
            return;
        }
        let Some(token) = self.access_token.clone() else {
            return;
        };
        let server_id = self.manifests.keys().copied().next();
        let instance_dir = match server_id {
            Some(ref id) => self.ctx.dirs.instance(id),
            None => self.ctx.dirs.root.clone(),
        };
        let ctx = self.ctx.clone();
        let master = self.ctx.config.get().master_url;
        tokio::spawn(async move {
            match crate::support::send_for_request(
                &ctx.http,
                &master,
                &token,
                &instance_dir,
                server_id,
                request_id,
            )
            .await
            {
                Ok(_) => ctx.send(MessageToFrontend::AddNotification {
                    key: "notif-support-sent".into(),
                    args: std::collections::BTreeMap::new(),
                    level: schema::NotifLevel::Info,
                }),
                Err(e) => tracing::warn!(error = %e, "requested logs were not sent"),
            }
        });
    }

    /// A refusal has to go back too: the master waits for an answer, and
    /// silence leaves the admin's page polling until the grant expires.
    fn answer_impersonate(&mut self, grant_id: Uuid, accepted: bool) {
        self.ctx
            .ws
            .send(ClientWsMsg::ImpersonateResponse { grant_id, accepted });
        if !accepted {
            return;
        }

        let Some(token) = self.access_token.clone() else {
            return;
        };
        // Keep our own token before the swap: leaving the other account means
        // going back to it, not signing in again.
        self.own_token = Some(token.clone());
        let ctx = self.ctx.clone();
        let master = self.ctx.config.get().master_url;
        let internal = self.ctx.internal.clone();
        tokio::spawn(async move {
            match crate::impersonation::claim(&ctx.http, &master, &token, grant_id).await {
                Ok(claimed) => {
                    let _ = internal.send(crate::backend::InternalEvent::ImpersonationStarted {
                        access_token: claimed.access_token,
                        username: claimed.username,
                    });
                }
                Err(e) => {
                    tracing::warn!(error = %e, "could not enter the player's account");
                    ctx.send(MessageToFrontend::AddNotification {
                        key: "notif-impersonate-failed".into(),
                        args: [("reason".to_string(), e.to_string())].into(),
                        level: schema::NotifLevel::Error,
                    });
                }
            }
        });
    }

    fn exit_impersonation(&mut self) {
        let Some(own) = self.own_token.take() else {
            return;
        };
        self.access_token = Some(own.clone());
        self.ctx.ws.set_token(Some(own));
        self.ctx
            .send(MessageToFrontend::ImpersonationChanged { as_username: None });
    }

    /// "Report a problem": collect the logs and send them to the master.
    ///
    /// Runs in the background, since collecting reads files off disk; the
    /// result comes back as a notification.
    fn send_support_bundle(&self, server_id: Option<Uuid>) {
        let Some(token) = self.access_token.clone() else {
            self.notify("notif-sign-in-first", schema::NotifLevel::Error);
            return;
        };
        // No server means no logs: the game writes them into the instance
        // directory.
        let Some(server_id) = server_id.or_else(|| self.manifests.keys().copied().next()) else {
            self.notify("notif-support-nothing-to-send", schema::NotifLevel::Warning);
            return;
        };

        let ctx = self.ctx.clone();
        let instance_dir = self.ctx.dirs.instance(&server_id);
        let master = self.ctx.config.get().master_url;
        tokio::spawn(async move {
            match crate::support::send(
                &ctx.http,
                &master,
                &token,
                &instance_dir,
                Some(server_id),
                "",
            )
            .await
            {
                Ok(id) => {
                    tracing::info!(%id, "support bundle sent");
                    ctx.send(MessageToFrontend::AddNotification {
                        key: "notif-support-sent".into(),
                        args: std::collections::BTreeMap::new(),
                        level: schema::NotifLevel::Info,
                    });
                }
                Err(e) => {
                    tracing::warn!(error = %e, "support bundle not sent");
                    ctx.send(MessageToFrontend::AddNotification {
                        key: "notif-support-failed".into(),
                        args: [("reason".to_string(), e.to_string())].into(),
                        level: schema::NotifLevel::Error,
                    });
                }
            }
        });
    }

    fn notify(&self, key: &str, level: schema::NotifLevel) {
        self.ctx.send(MessageToFrontend::AddNotification {
            key: key.into(),
            args: std::collections::BTreeMap::new(),
            level,
        });
    }
}

/// Switches the skin model. Answers with the updated profile, same as an upload.
async fn set_skin_model_on_master(
    http: &reqwest::Client,
    master: &str,
    token: &str,
    slim: bool,
) -> Result<schema::UserProfile, String> {
    let url = format!("{}/api/me/skin/model", master.trim_end_matches('/'));
    let res = http
        .put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "model": if slim { "slim" } else { "classic" } }))
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
