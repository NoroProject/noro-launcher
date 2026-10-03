// Over 150 lines: everything the player does to a server: open, launch, stop,
// pick mods and builds.
//! The server list, launching and stopping games, optional mods and builds.

use super::*;

impl LauncherUI {
    /// By reference: the sidebar asks for every server on every frame, and a
    /// copy each time duplicated the stage map and the rate samples.
    pub fn sync_state(&self, server_id: &Uuid) -> &SyncUiState {
        static IDLE: std::sync::LazyLock<SyncUiState> =
            std::sync::LazyLock::new(SyncUiState::default);
        self.sync.get(server_id).unwrap_or(&IDLE)
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

    pub(super) fn replace_servers(&mut self, servers: Vec<ServerEntry>, cx: &mut Context<Self>) {
        let next_ids: HashSet<_> = servers.iter().map(|s| s.id).collect();
        let old_ids: Vec<_> = self.servers.iter().map(|s| s.id).collect();

        for id in old_ids {
            if !next_ids.contains(&id) {
                self.clear_background(id, cx);
                self.clear_icon(id, cx);
            }
        }

        for server in &servers {
            if trimmed(server.background_url.as_ref()).is_none() {
                self.clear_background(server.id, cx);
            }
            if trimmed(server.icon_url.as_ref()).is_none() {
                self.clear_icon(server.id, cx);
            }
        }
        // A changed address needs nothing here: `ensure_*_loaded` sees the new
        // URL on the next frame, loads it, and releases the old picture when
        // the new one replaces it.

        self.servers = servers;

        // Back where the player left off, once, on the first list.
        if !self.last_server_restored {
            self.last_server_restored = true;
            let saved = self.ui_state.last_server;
            if self.page == Page::Servers {
                if let Some(id) = saved.filter(|id| self.servers.iter().any(|s| &s.id == id)) {
                    self.page = Page::ServerDetail(id);
                }
            }
        }
    }

    /// Remembers the open server for the next start. Cheap enough for every
    /// frame: it only compares.
    pub fn note_open_server(&mut self) {
        let open = match self.page {
            Page::ServerDetail(id)
            | Page::ServerMods(id)
            | Page::ServerModCatalog(id)
            | Page::ServerSettings(id) => Some(id),
            _ => None,
        };
        if open.is_some() && open != self.ui_state.last_server {
            self.ui_state.last_server = open;
        }
    }

    pub fn open_server(&mut self, id: Uuid) {
        self.page = Page::ServerDetail(id);
        self.backend
            .send(MessageToBackend::OpenServer { server_id: id });
    }

    pub fn launch(&mut self, id: Uuid) {
        let modal = bridge::ModalAction::new("Launch");
        let s = self.sync.entry(id).or_default();
        s.syncing = true;
        s.failed = None;
        s.heading = Some(SyncHeading::Preparing);
        s.launch = Some(modal.clone());
        self.backend.send(MessageToBackend::LaunchServer {
            server_id: id,
            modal_action: modal,
        });
    }

    /// Asks the running sync to stop. It stops between chunks and reports back
    /// with `LaunchCancelled`; the button says "cancelling" until then, so a
    /// new launch can't start under the old one.
    pub fn cancel_launch(&mut self, id: Uuid) {
        if let Some(s) = self.sync.get_mut(&id) {
            if let Some(modal) = &s.launch {
                modal.cancel();
                s.heading = Some(SyncHeading::Cancelling);
            }
        }
    }

    /// Two clicks for anything that can't be undone: the first arms the button
    /// for a few seconds, the second goes through. True on the second.
    pub fn confirm_or_arm(&mut self, key: String, cx: &mut Context<Self>) -> bool {
        if self.armed_action.as_deref() == Some(key.as_str()) {
            self.armed_action = None;
            return true;
        }
        self.armed_action = Some(key.clone());
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            executor.timer(std::time::Duration::from_secs(4)).await;
            let _ = this.update(cx, |ui, cx| {
                if ui.armed_action.as_deref() == Some(key.as_str()) {
                    ui.armed_action = None;
                    cx.notify();
                }
            });
        })
        .detach();
        false
    }

    pub fn is_armed(&self, key: &str) -> bool {
        self.armed_action.as_deref() == Some(key)
    }

    /// First click arms, second click stops; the arming wears off on its own.
    pub fn stop_clicked(&mut self, id: Uuid, cx: &mut Context<Self>) {
        let s = self.sync.entry(id).or_default();
        if s.stop_armed {
            s.stop_armed = false;
            self.kill(id);
            return;
        }
        s.stop_armed = true;
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            executor.timer(std::time::Duration::from_secs(4)).await;
            let _ = this.update(cx, |ui, cx| {
                if let Some(s) = ui.sync.get_mut(&id) {
                    s.stop_armed = false;
                }
                cx.notify();
            });
        })
        .detach();
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
    pub(super) fn blocking_issue(
        mods: &[OptionalModInfo],
        name: &str,
    ) -> Option<(String, NotifLevel)> {
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
    pub(super) fn as_rule(m: &OptionalModInfo) -> schema::build::OptionalMod {
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
}
