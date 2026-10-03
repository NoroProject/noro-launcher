//! Window requests about servers, launches, optional mods, builds and Java runtimes.

use super::*;

impl BackendState {
    /// Only ever gets the variants `handle_to_backend` routes here.
    pub(super) async fn on_server_request(&mut self, msg: MessageToBackend) {
        match msg {
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
                let Some(api) = crate::master_api::MasterApi::for_session(&self.ctx) else {
                    self.notify("notif-sign-in-to-suggest", schema::NotifLevel::Error);
                    return;
                };
                let ctx = self.ctx.clone();
                tokio::spawn(async move {
                    let body = serde_json::json!({
                        "server_id": server_id,
                        "build_id": build_id,
                        "provider": provider,
                        "project_id": project_id,
                        "title": title,
                        "icon_url": icon_url,
                        "description": description,
                    });
                    match api.suggest_mod(&body).await {
                        Ok(()) => ctx.send(MessageToFrontend::AddNotification {
                            key: "notif-mod-suggestion-sent".into(),
                            args: std::collections::BTreeMap::new(),
                            level: schema::NotifLevel::Info,
                        }),
                        Err(e) => ctx.send(MessageToFrontend::AddNotification {
                            key: "notif-mod-suggestion-failed".into(),
                            args: [("reason".to_string(), format!("{e:#}"))].into(),
                            level: schema::NotifLevel::Error,
                        }),
                    }
                });
            }
            MessageToBackend::RequestJavaRuntimes { server_id } => {
                crate::personal::request_java(&self.ctx, server_id)
            }
            MessageToBackend::SetJavaRuntime {
                server_id,
                component,
            } => crate::personal::set_java(&self.ctx, server_id, component),
            _ => {}
        }
    }
}
