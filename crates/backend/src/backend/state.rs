// Over 150 lines: the main loop and the internal events it handles; each event
// changes the state this file owns.
//! The backend's main loop and the state it owns: the session, the
//! servers, internal events.

use super::*;

impl BackendState {
    pub(super) async fn main_loop(&mut self) {
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
                    self.online = online;
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
                self.set_user(user.clone());
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
                self.set_user(user.clone());
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
                self.set_user(user.clone());
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
                if self.user.is_some() {
                    return;
                }
                // The master can't be reached, which says nothing against the
                // session. Carry on as the player from last time, so installed
                // builds can still be started; the socket checks the token
                // properly once the master is back.
                match (
                    &self.access_token,
                    crate::offline_cache::load_profile(&self.ctx.dirs),
                ) {
                    (Some(_), Some(user)) => {
                        tracing::info!("master unreachable, continuing with the cached profile");
                        self.set_user(user.clone());
                        self.ctx.send(MessageToFrontend::LoginSuccess { user });
                    }
                    _ => self.ctx.send(MessageToFrontend::SessionCheckDone),
                }
            }
            InternalEvent::JarIconsReady { server_id } => {
                if let Some(manifest) = self.manifests.get(&server_id).cloned() {
                    self.send_optional_mods(server_id, &manifest);
                }
            }
            InternalEvent::ManifestTimeout { server_id, seq } => {
                if self.launch_seq.get(&server_id) != Some(&seq) {
                    return;
                }
                // The master went quiet; an installed build can still start
                // from what it sent last time.
                if let Some(manifest) = self.cached_manifests.get(&server_id).cloned() {
                    if self.pending_launch.contains_key(&server_id) {
                        tracing::info!(%server_id, "no manifest from the master, launching the installed build");
                        self.begin_launch(server_id, manifest);
                        return;
                    }
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
        crate::offline_cache::forget_account(&self.ctx.dirs);
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

    /// Our own profile is also kept on disk for starting without the master; a
    /// borrowed one isn't, like the borrowed token.
    pub fn set_user(&mut self, user: UserProfile) {
        if self.own_token.is_none() {
            crate::offline_cache::save_profile(&self.ctx.dirs, &user);
        }
        self.ctx.set_profile(Some(user.clone()));
        self.user = Some(user);
    }

    /// The disk tells whether a build is installed until its manifest comes.
    /// Without this every installed build offered «Install» until the master
    /// answered.
    pub fn announce_installed_states(&mut self, servers: &[ServerEntry]) {
        for server in servers {
            if self.build_state_known.insert(server.id) {
                self.ctx.send(MessageToFrontend::BuildStateChanged {
                    server_id: server.id,
                    state: crate::sync::installed_state(&self.ctx.dirs.instance(&server.id)),
                });
            }
        }
    }

    /// The last server list and installed builds' manifests, before the
    /// master answers or in case it never does.
    pub(super) fn load_offline_cache(&mut self) {
        let servers = crate::offline_cache::load_servers(&self.ctx.dirs);
        if servers.is_empty() {
            return;
        }
        for server in &servers {
            if let Some(manifest) = crate::offline_cache::load_manifest(&self.ctx.dirs, &server.id)
            {
                self.cached_manifests.insert(server.id, manifest);
            }
        }
        self.announce_installed_states(&servers);
        self.servers = servers.clone();
        self.ctx.send(MessageToFrontend::ServerList { servers });
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
