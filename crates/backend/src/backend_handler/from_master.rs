// Over 150 lines: one match over everything the master pushes. Split, a new
// message could end up handled twice or not at all.
//! What the master pushes over the socket.

use super::*;

impl BackendState {
    pub async fn handle_from_master(&mut self, msg: ServerWsMsg) {
        match msg {
            ServerWsMsg::AuthOk { user } => {
                self.set_user(user.clone());
                // The window asks for the skin presets itself when the login
                // lands; fetching them here as well downloaded every preset
                // twice per login and again on every reconnect.
                self.ctx.send(MessageToFrontend::LoginSuccess { user });
            }
            // The feed and the counter live on the master; all that is left is to show them.
            ServerWsMsg::NotificationPush {
                notification,
                unread,
                os_toast,
            } => crate::notifications::arrived(&self.ctx, notification, unread, os_toast),
            // The set may have changed on another of the player's machines.
            ServerWsMsg::PersonalContentChanged { server_id } => {
                crate::personal::request(&self.ctx, server_id)
            }
            ServerWsMsg::DirectMessage { message } => {
                let mine = self.user.as_ref().map(|u| u.id) == Some(message.author_id);
                self.ctx.send(MessageToFrontend::DmArrived {
                    // The peer is not the author: your own message arriving from another
                    // client belongs to the conversation with its recipient.
                    peer: message.author_id,
                    message: bridge::DmMessageView {
                        author_name: message.author_name,
                        body: message.body,
                        at: message.at.timestamp(),
                        mine,
                    },
                });
            }

            ServerWsMsg::AuthFail { reason } => {
                tracing::warn!("auth fail: {reason}");
                self.on_auth_failed();
            }
            ServerWsMsg::ServerList { servers } => {
                self.announce_installed_states(&servers);
                // A borrowed account's list stays out of the cache, like its
                // profile: it may hold servers only their roles can see.
                if self.own_token.is_none() {
                    crate::offline_cache::save_servers(&self.ctx.dirs, &servers);
                }
                self.servers = servers.clone();
                self.ctx.send(MessageToFrontend::ServerList { servers });
            }
            ServerWsMsg::News { items } => {
                self.ctx.send(MessageToFrontend::NewsUpdated { items });
            }
            ServerWsMsg::BuildManifest { manifest } => {
                let server_id = manifest.server_id;
                crate::offline_cache::save_manifest(&self.ctx.dirs, &manifest);
                self.cached_manifests.insert(server_id, manifest.clone());
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
                // The master's answer to a manifest request it won't serve. It
                // names no server, but a launch waiting on a manifest waits on
                // exactly this, and left pending it kept «Preparing» up for good.
                if matches!(
                    key.as_str(),
                    "notif-no-published-build" | "notif-build-pending-import"
                ) {
                    for (server_id, modal) in self.pending_launch.drain() {
                        modal.fail(&key);
                        self.ctx
                            .send(MessageToFrontend::LaunchCancelled { server_id });
                    }
                }
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
                self.set_user(user.clone());
                self.ctx
                    .send(MessageToFrontend::PermissionsUpdated { user });
                self.ctx.ws.send(ClientWsMsg::RequestServerList);
                // Taken out for the loop rather than cloned: manifests are big.
                let manifests = std::mem::take(&mut self.manifests);
                for (server_id, manifest) in &manifests {
                    self.send_optional_mods(*server_id, manifest);
                }
                self.manifests = manifests;
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
}
