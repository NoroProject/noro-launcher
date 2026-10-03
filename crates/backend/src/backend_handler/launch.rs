//! Launching a build and telling the window about builds: state, optional
//! mods, recommended settings.

use super::*;

impl BackendState {
    pub(super) async fn launch_server(&mut self, server_id: Uuid, modal: bridge::ModalAction) {
        if self.login_info().is_none() {
            modal.fail(crate::failure::NOT_SIGNED_IN);
            self.ctx.send(MessageToFrontend::SyncFailed {
                server_id,
                reason: crate::failure::NOT_SIGNED_IN.into(),
                detail: String::new(),
            });
            return;
        }
        if self.ctx.running.lock().contains_key(&server_id) {
            self.ctx.send(MessageToFrontend::AddNotification {
                key: "notif-already-running".into(),
                args: Default::default(),
                level: schema::NotifLevel::Warning,
            });
            // The window thought the game was down, or it wouldn't have offered
            // a launch. Tell it otherwise, or the button sits on «Preparing».
            self.ctx.send(MessageToFrontend::GameStarted { server_id });
            return;
        }

        self.last_launched = Some(server_id);
        self.pending_launch.insert(server_id, modal);

        let offline_manifest = if self.online {
            None
        } else {
            self.cached_manifests.get(&server_id).cloned()
        };
        if let Some(manifest) = self.manifests.get(&server_id).cloned() {
            self.begin_launch(server_id, manifest);
        } else if let Some(manifest) = offline_manifest {
            // No master to ask; the build on disk is what there is.
            tracing::info!(%server_id, "offline, launching the installed build");
            self.begin_launch(server_id, manifest);
        } else {
            self.ctx.ws.send(self.request_manifest_msg(server_id));
            // The answer can be lost with the socket. Without a deadline the
            // button stayed on «Preparing» for good.
            let seq = {
                let seq = self.launch_seq.entry(server_id).or_default();
                *seq += 1;
                *seq
            };
            let internal = self.ctx.internal.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(45)).await;
                let _ = internal.send(InternalEvent::ManifestTimeout { server_id, seq });
            });
        }
    }

    /// Sync and launch for a manifest that is already in hand.
    pub(crate) fn begin_launch(&mut self, server_id: Uuid, manifest: schema::BuildManifest) {
        let Some(modal) = self.pending_launch.remove(&server_id) else {
            return; // nobody asked for a launch
        };
        let (Some(login), Some(user)) = (self.login_info(), self.user.clone()) else {
            modal.fail(crate::failure::NOT_SIGNED_IN);
            self.ctx.send(MessageToFrontend::SyncFailed {
                server_id,
                reason: crate::failure::NOT_SIGNED_IN.into(),
                detail: String::new(),
            });
            return;
        };
        let enabled = crate::sync::file_sync::resolve_enabled(
            &manifest,
            self.ctx.optional.get().for_server(&server_id),
        );
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
            discord_rpc: c.discord_rpc,
            master_url: c.master_url,
            server_settings,
            system_memory_mb: crate::system::total_memory_mb(),
        });
    }

    /// Update packs and shaders without leaving the game.
    ///
    /// Silent when there is nothing to update: a "synced" toast for every
    /// little thing teaches the player to ignore it.
    pub(super) fn live_sync(&self, server_id: uuid::Uuid, manifest: schema::BuildManifest) {
        // Without a profile there is no telling which limited packs are this
        // player's to have.
        let Some(user) = self.user.clone() else {
            return;
        };
        let enabled = crate::sync::file_sync::resolve_enabled(
            &manifest,
            self.ctx.optional.get().for_server(&server_id),
        );
        let dir = self.ctx.dirs.instance(&server_id);
        let client = self.ctx.http.clone();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match crate::sync::live::apply(&client, &dir, &manifest, &enabled, &user).await {
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
                Err(e) => tracing::warn!(error = %format!("{e:#}"), "live sync failed"),
            }
        });
    }

    /// Tells the frontend what can be done with the build right now.
    pub(super) fn send_build_state(
        &mut self,
        server_id: uuid::Uuid,
        manifest: &schema::BuildManifest,
    ) {
        self.build_state_known.insert(server_id);
        let dir = self.ctx.dirs.instance(&server_id);
        self.ctx.send(bridge::MessageToFrontend::BuildStateChanged {
            server_id,
            state: crate::sync::build_state(&dir, manifest),
        });
    }

    pub(crate) fn send_optional_mods(&mut self, server_id: Uuid, manifest: &schema::BuildManifest) {
        use crate::directories::safe_join;
        // Jars nobody has read yet are read off the loop; the list goes out
        // again once they are, with their icons.
        let mut unread: Vec<std::path::PathBuf> = Vec::new();
        let enabled = crate::sync::file_sync::resolve_enabled(
            manifest,
            self.ctx.optional.get().for_server(&server_id),
        );
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
                let is_enabled = enabled.contains(&m.name);
                let icon_url = m.icon_url.clone().or_else(|| {
                    m.files
                        .iter()
                        .filter(|f| f.ends_with(".jar"))
                        .find_map(|f| {
                            let path = safe_join(&instance_dir, f)?;
                            match crate::mod_icon::known_jar_icon(&path) {
                                Some(icon) => icon,
                                None => {
                                    unread.push(path);
                                    None
                                }
                            }
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
        let installed_files = if self.files_sent_for.get(&server_id) == Some(&manifest.build_id) {
            None
        } else {
            self.files_sent_for.insert(server_id, manifest.build_id);
            Some(
                manifest
                    .verified_files
                    .iter()
                    .map(|f| f.path.clone())
                    .collect(),
            )
        };
        if !unread.is_empty() {
            let internal = self.ctx.internal.clone();
            tokio::task::spawn_blocking(move || {
                for path in &unread {
                    crate::mod_icon::cached_jar_icon(path);
                }
                let _ = internal.send(InternalEvent::JarIconsReady { server_id });
            });
        }
        self.ctx.send(MessageToFrontend::OptionalMods {
            server_id,
            mods,
            allow_suggestions: manifest.allow_optional_mod_suggestions,
            allow_personal: manifest.allow_personal_content,
            installed_files,
        });
    }

    pub(super) fn send_server_recommendation(
        &self,
        server_id: Uuid,
        manifest: &schema::BuildManifest,
    ) {
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
}
