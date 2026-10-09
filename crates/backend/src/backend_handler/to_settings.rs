//! Window requests about launcher and per-server settings.

use super::*;

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
                self.ctx.rpc.set_enabled(enabled);
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
            _ => {}
        }
    }
}
