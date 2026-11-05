//! Window requests about launcher and per-server settings.

use super::*;
use crate::screenshots;

impl BackendState {
    /// Only ever gets the variants `handle_to_backend` routes here.
    pub(super) async fn on_settings_request(&mut self, msg: MessageToBackend) {
        match msg {
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
            MessageToBackend::SetConsoleSettings { settings } => {
                self.ctx.config.update(|c| c.console = settings);
            }
            MessageToBackend::SetCrashReports { enabled } => {
                // Takes effect on the next start: Sentry comes up before GPUI,
                // and an installed panic hook can't be taken back off.
                self.ctx.config.update(|c| c.crash_reports = enabled);
                self.send_config_state();
            }
            MessageToBackend::SetDiscordRpc { enabled } => {
                self.ctx.config.update(|c| c.discord_rpc = enabled);
                self.ctx
                    .rpc
                    .set_enabled(crate::streamer::rpc_allowed(&self.ctx));
            }
            MessageToBackend::SetStreamerMode { on } => crate::streamer::set_manual(&self.ctx, on),
            MessageToBackend::SetStreamerAuto { enabled } => {
                crate::streamer::set_auto(&self.ctx, enabled)
            }
            MessageToBackend::SetHideWhilePlaying { enabled } => {
                self.ctx.config.update(|c| c.hide_while_playing = enabled);
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
            MessageToBackend::SetServerAutoJoin { server_id, choice } => {
                self.ctx
                    .config
                    .update(|c| c.set_server_auto_join(server_id, choice));
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
            MessageToBackend::RequestScreenshots { server_id } => {
                self.send_screenshots(server_id);
            }
            MessageToBackend::OpenScreenshot {
                server_id,
                file_name,
            } => {
                let folder = screenshots::folder(&self.ctx.dirs.instance(&server_id));
                if let Some(path) = screenshots::resolve(&folder, &file_name) {
                    let _ = open::that(path);
                }
            }
            MessageToBackend::DeleteScreenshot {
                server_id,
                file_name,
            } => {
                let folder = screenshots::folder(&self.ctx.dirs.instance(&server_id));
                if let Some(path) = screenshots::resolve(&folder, &file_name) {
                    if let Err(e) = std::fs::remove_file(&path) {
                        tracing::warn!(error = %e, file = %file_name, "screenshot not deleted");
                    }
                }
                self.send_screenshots(server_id);
            }
            MessageToBackend::RepairServerFiles { server_id } => {
                self.repair_server_files(server_id);
            }
            MessageToBackend::OpenScreenshotsFolder { server_id } => {
                let folder = screenshots::folder(&self.ctx.dirs.instance(&server_id));
                let _ = std::fs::create_dir_all(&folder);
                let _ = open::that(folder);
            }
            _ => {}
        }
    }

    /// Reading a folder of a few hundred entries is quick, but it is still disk
    /// work: off the handler loop, like everything else that touches files.
    fn send_screenshots(&self, server_id: Uuid) {
        let folder = screenshots::folder(&self.ctx.dirs.instance(&server_id));
        let ctx = self.ctx.clone();
        tokio::task::spawn_blocking(move || {
            let items = screenshots::list(&folder);
            ctx.send(MessageToFrontend::Screenshots { server_id, items });
        });
    }

    /// The same reinstall an admin can ask for, started by the player: most
    /// "the game won't start" cases are a half-written or foreign mod, and
    /// waiting for staff to press a button for it helped nobody.
    fn repair_server_files(&self, server_id: Uuid) {
        // Deleting mods under a running JVM breaks the session and leaves the
        // files half gone.
        if self.ctx.running.lock().contains_key(&server_id) {
            self.notify("notif-repair-busy", schema::NotifLevel::Warning);
            return;
        }
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            let action = schema::RemoteAction::ReinstallBuild;
            match crate::remote_actions::run(&ctx.dirs, action, Some(server_id)).await {
                Ok(outcome) => {
                    tracing::info!(%server_id, "player repair: {}", outcome.message);
                    ctx.send(MessageToFrontend::AddNotification {
                        key: "notif-repair-done".into(),
                        args: Default::default(),
                        level: schema::NotifLevel::Info,
                    });
                }
                Err(e) => tracing::warn!(error = %format!("{e:#}"), "player repair failed"),
            }
        });
    }
}
