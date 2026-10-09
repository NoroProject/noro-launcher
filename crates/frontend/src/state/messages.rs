//! Messages from the backend, routed by what they are about. The match here
//! names every variant, so a new one doesn't compile until it has a place.

use super::*;

impl LauncherUI {
    pub fn on_message(&mut self, msg: MessageToFrontend, cx: &mut Context<Self>) {
        use MessageToFrontend as M;
        match msg {
            M::GameLog { server_id, lines } => {
                self.on_game_log(server_id, lines, cx);
                // Nothing in the main window shows the log; redrawing it for
                // every batch kept the launcher busy for as long as the game
                // was writing.
                return;
            }
            m @ (M::LoginSuccess { .. }
            | M::LoginFailed { .. }
            | M::LoggedOut
            | M::SessionCheckDone
            | M::ConfigState { .. }
            | M::LocaleCatalog { .. }
            | M::ConnectionState { .. }
            | M::OpenOrFocusMainWindow
            | M::CloseModal
            | M::Quit
            | M::LauncherUpdateAvailable { .. }
            | M::AddNotification { .. }
            | M::ImpersonatePrompt { .. }
            | M::LogRequestPrompt { .. }
            | M::RemoteActionPrompt { .. }
            | M::ImpersonationChanged { .. }
            | M::PermissionsUpdated { .. }) => self.on_session_message(m, cx),
            m @ (M::ServerList { .. }
            | M::OptionalMods { .. }
            | M::ServerClientRecommendation { .. }
            | M::SyncProgress { .. }
            | M::SyncComplete { .. }
            | M::LaunchStep { .. }
            | M::LaunchCancelled { .. }
            | M::LiveSynced { .. }
            | M::SyncFailed { .. }
            | M::GameStarted { .. }
            | M::GameStopped { .. }
            | M::BuildStateChanged { .. }
            | M::JavaRuntimes { .. }) => self.on_sync_message(m, cx),
            m @ (M::NewsUpdated { .. }
            | M::CatalogSearchResults { .. }
            | M::CatalogFailed { .. }
            | M::ModProjectLoaded { .. }
            | M::PersonalContent { .. }
            | M::ContentVersions { .. }
            | M::ContentActionFailed { .. }
            | M::SkinUploadFailed
            | M::CapesList { .. }
            | M::SkinPresetsList { .. }) => self.on_content_message(m, cx),
            m @ (M::NotificationFeed { .. }
            | M::NotificationArrived { .. }
            | M::UnreadChanged { .. }
            | M::PunishmentsLoaded { .. }
            | M::RulesLoaded { .. }
            | M::TicketsLoaded { .. }
            | M::TicketLoaded { .. }
            | M::DmThreadsLoaded { .. }
            | M::DmThreadLoaded { .. }
            | M::DmArrived { .. }) => self.on_account_message(m),
        }
        cx.notify();
    }
}
