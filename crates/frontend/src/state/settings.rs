// Over 150 lines: a setter for each setting; each is a few lines.
//! Launcher and per-server settings, the language and the self-update.

use super::*;

impl LauncherUI {
    /// The most the memory steppers go to: this computer's RAM less a gigabyte
    /// for the system, or the old fixed cap where the RAM isn't known. The
    /// steppers used to go to 64 GB on any machine.
    pub fn memory_ceiling_mb(&self) -> u32 {
        self.config
            .system_memory_mb
            .map(|m| m.saturating_sub(1024).max(1024))
            .unwrap_or(65536)
            .min(65536)
    }

    /// The game's share past three quarters of the RAM leaves the system and
    /// the launcher swapping, which looks like the game lagging.
    pub fn memory_warning(&self, max_mb: u32) -> Option<String> {
        let total = self.config.system_memory_mb?;
        if (max_mb as u64) * 4 <= (total as u64) * 3 {
            return None;
        }
        let mut args = i18n::FluentArgs::new();
        args.set("total", format!("{:.0}", total as f64 / 1024.0));
        Some(i18n::t_args("settings-memory-too-much", &args))
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

    pub fn set_discord_rpc(&mut self, enabled: bool) {
        self.config.discord_rpc = enabled;
        self.backend
            .send(MessageToBackend::SetDiscordRpc { enabled });
    }

    /// Deferred, like raising the window on a second launch: this runs inside
    /// an update of the window's own view, and the window can't be updated
    /// from there.
    pub(super) fn set_main_window_minimized(&self, minimized: bool, cx: &mut Context<Self>) {
        let Some(handle) = self.main_window else {
            return;
        };
        cx.defer(move |cx| {
            let _ = handle.update(cx, |_, window, _| {
                if minimized {
                    window.minimize_window();
                } else {
                    window.activate_window();
                }
            });
            if !minimized {
                cx.activate(true);
            }
        });
    }

    pub fn set_hide_while_playing(&mut self, enabled: bool) {
        self.config.hide_while_playing = enabled;
        self.backend
            .send(MessageToBackend::SetHideWhilePlaying { enabled });
    }

    /// Takes effect on the next launch: Sentry comes up before GPUI, and its
    /// panic hook can't be removed while the process runs.
    pub fn set_crash_reports(&mut self, enabled: bool) {
        self.config.crash_reports = enabled;
        self.backend
            .send(MessageToBackend::SetCrashReports { enabled });
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

    /// `None` goes back to joining the build's first entry point.
    pub fn set_server_auto_join(&mut self, server_id: Uuid, choice: Option<bridge::AutoJoin>) {
        match choice {
            Some(choice) => {
                self.config.auto_join.insert(server_id, choice);
            }
            None => {
                self.config.auto_join.remove(&server_id);
            }
        }
        self.backend
            .send(MessageToBackend::SetServerAutoJoin { server_id, choice });
    }

    pub fn reset_server_client_settings(&mut self, server_id: Uuid) {
        self.server_settings.remove(&server_id);
        self.config.auto_join.remove(&server_id);
        self.backend
            .send(MessageToBackend::ResetServerClientSettings { server_id });
    }

    /// A download in progress is writing the same files; the backend only
    /// knows about a running game, so the window refuses the sync itself.
    pub fn repair_server_files(&mut self, server_id: Uuid, cx: &mut Context<Self>) {
        if self.sync.get(&server_id).is_some_and(|s| s.syncing) {
            self.notify_toast(
                i18n::t("notif-repair-busy"),
                schema::NotifLevel::Warning,
                cx,
            );
            return;
        }
        self.backend
            .send(MessageToBackend::RepairServerFiles { server_id });
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

    pub fn install_update(&mut self, cx: &mut Context<Self>) {
        if self.updating {
            return;
        }
        let Some(v) = self.update_available.clone() else {
            return;
        };
        self.updating = true;
        let modal = bridge::ModalAction::new("Update");
        self.update_modal = Some(modal.clone());
        self.backend.send(MessageToBackend::InstallUpdate {
            version: v,
            modal_action: modal.clone(),
        });
        // The download reports into the modal, not through messages: redraw a
        // few times a second while it runs, and stand down if it fails. On
        // success the launcher restarts into the new version.
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| loop {
            executor.timer(std::time::Duration::from_millis(250)).await;
            let progress = modal.snapshot();
            let done = progress.error.is_some() || progress.finished;
            let alive = this.update(cx, |ui, cx| {
                if progress.error.is_some() {
                    ui.updating = false;
                    ui.update_modal = None;
                }
                cx.notify();
            });
            if done || alive.is_err() {
                break;
            }
        })
        .detach();
    }
}
