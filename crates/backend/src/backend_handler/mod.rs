//! Message handling: `MessageToBackend` from the frontend, `ServerWsMsg` from
//! the master.

use std::path::PathBuf;

use crate::auth::web_login;
use crate::backend::{spawn_sync_and_launch, BackendState, InternalEvent};
use bridge::{
    ClientSettingsState, LoginErrorKind, MessageToBackend, MessageToFrontend, OptionalModInfo,
};
use schema::{ClientWsMsg, ServerWsMsg};
use uuid::Uuid;

mod from_master;
mod launch;
mod staff;
mod to_account;
mod to_content;
mod to_servers;
mod to_session;
mod to_settings;

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

    /// Requests from the window, routed by what they are about. Every variant
    /// is named here, so a new one doesn't compile until it has a place.
    pub async fn handle_to_backend(&mut self, msg: MessageToBackend) {
        use MessageToBackend as M;
        match msg {
            m @ (M::StartWebLogin { .. }
            | M::Logout
            | M::RemoteActionAnswer { .. }
            | M::LogRequestAnswer { .. }
            | M::ImpersonateAnswer { .. }
            | M::ImpersonateExit
            | M::SendSupportBundle { .. }
            | M::InstallUpdate { .. }
            | M::SetLocale { .. }
            | M::FocusWindow
            | M::Quit) => self.on_session_request(m).await,
            m @ (M::RequestServerList
            | M::RequestNews
            | M::OpenServer { .. }
            | M::LaunchServer { .. }
            | M::KillGame { .. }
            | M::SetOptionalMods { .. }
            | M::SelectBuild { .. }
            | M::SuggestOptionalMod { .. }
            | M::RequestJavaRuntimes { .. }
            | M::SetJavaRuntime { .. }) => self.on_server_request(m).await,
            m @ (M::SetMemory { .. }
            | M::SetJvmFlags { .. }
            | M::SetShowConsoleOnLaunch { .. }
            | M::SetFullscreen { .. }
            | M::SetCrashReports { .. }
            | M::SetConsoleSettings { .. }
            | M::SetDiscordRpc { .. }
            | M::SetServerMemory { .. }
            | M::SetServerJvmFlags { .. }
            | M::SetServerShowConsoleOnLaunch { .. }
            | M::SetServerFullscreen { .. }
            | M::ResetServerClientSettings { .. }
            | M::OpenServerClientFolder { .. }) => self.on_settings_request(m).await,
            m @ (M::SearchCatalog { .. }
            | M::RequestModProject { .. }
            | M::RequestPersonalContent { .. }
            | M::RequestContentVersions { .. }
            | M::InstallPersonalContent { .. }
            | M::RemovePersonalContent { .. }
            | M::SetPersonalContentEnabled { .. }) => self.on_content_request(m).await,
            m @ (M::SetSkinModel { .. }
            | M::UploadSkin { .. }
            | M::RequestCapesList
            | M::RequestSkinPresetsList
            | M::SelectCape { .. }
            | M::RequestNotifications { .. }
            | M::MarkNotificationRead { .. }
            | M::MarkAllNotificationsRead
            | M::RequestPunishments
            | M::RequestRules
            | M::RequestTickets
            | M::RequestTicket { .. }
            | M::OpenTicket { .. }
            | M::ReplyTicket { .. }
            | M::RequestDmThreads
            | M::RequestDmThread { .. }
            | M::SendDm { .. }) => self.on_account_request(m).await,
        }
    }

    fn request_skin_presets(&self) {
        let Some(api) = crate::master_api::MasterApi::for_session(&self.ctx) else {
            return;
        };
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match api.skin_presets().await {
                Ok(presets) => ctx.send(MessageToFrontend::SkinPresetsList { presets }),
                Err(e) => tracing::warn!(error = %format!("{e:#}"), "skin presets did not load"),
            }
        });
    }

    /// Whose logs to send when the request names no server: the one last
    /// launched, else the instance whose game wrote a log most recently. Picking
    /// the first key of a `HashMap` sent an arbitrary server's logs.
    fn fallback_log_server(&self) -> Option<Uuid> {
        if self.last_launched.is_some() {
            return self.last_launched;
        }
        let entries = std::fs::read_dir(self.ctx.dirs.instances()).ok()?;
        entries
            .filter_map(Result::ok)
            .filter_map(|e| {
                let id = Uuid::parse_str(&e.file_name().to_string_lossy()).ok()?;
                let log = e.path().join("logs/latest.log");
                let modified = std::fs::metadata(&log).and_then(|m| m.modified()).ok()?;
                Some((modified, id))
            })
            .max()
            .map(|(_, id)| id)
    }

    fn notify(&self, key: &str, level: schema::NotifLevel) {
        self.ctx.send(MessageToFrontend::AddNotification {
            key: key.into(),
            args: std::collections::BTreeMap::new(),
            level,
        });
    }
}
