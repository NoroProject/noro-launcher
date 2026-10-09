// Over 150 lines: one handler for every session message; split, the match would
// only move.
//! Backend messages about sign-in, settings, connection and requests from staff.

use super::*;

impl LauncherUI {
    /// Only ever gets the variants `on_message` routes here.
    pub(super) fn on_session_message(&mut self, msg: MessageToFrontend, cx: &mut Context<Self>) {
        match msg {
            MessageToFrontend::LoginSuccess { user } => {
                // The bell counter has to be right before the panel is opened:
                // otherwise anything unread that arrived while offline would
                // never show.
                self.backend.send(MessageToBackend::RequestNotifications {
                    offset: 0,
                    unread_only: false,
                });
                self.user = Some(user);
                self.load_user_skin(cx);
                self.logging_in = false;
                self.login_modal = None;
                self.startup_checking = false;
                self.login_error = None;
                self.backend.send(MessageToBackend::RequestCapesList);
                self.backend.send(MessageToBackend::RequestSkinPresetsList);
                if self.page == Page::Login {
                    self.page = Page::Servers;
                }
            }
            MessageToFrontend::LoginFailed { kind } => {
                self.logging_in = false;
                self.login_modal = None;
                self.startup_checking = false;
                self.login_error = Some(match kind {
                    LoginErrorKind::Cancelled => i18n::t("error-sign-in-cancelled"),
                    // `r` is a translation key from the master, not text.
                    LoginErrorKind::Rejected(r) => i18n::t(&r),
                    LoginErrorKind::Network(e) => {
                        let mut args = i18n::FluentArgs::new();
                        args.set("reason", e);
                        i18n::t_args("error-network", &args)
                    }
                });
            }
            MessageToFrontend::LoggedOut => {
                self.clear_account_data();
                self.user = None;
                self.reset_skin_preview(cx);
                self.skin_url = None;
                self.skin_loading = false;
                self.skin_uploading = false;
                self.skin_dragging = false;
                self.skin_anim_running = false;
                self.cape_bytes = None;
                self.cape_url = None;
                self.cape_loading = false;
                self.avatar_image = None;
                self.avatar_loading = false;
                self.logging_in = false;
                self.startup_checking = false;
                self.page = Page::Login;
            }
            MessageToFrontend::SessionCheckDone => {
                self.startup_checking = false;
            }
            MessageToFrontend::ConfigState {
                memory_min_mb,
                memory_max_mb,
                jvm_flags,
                show_console_on_launch,
                fullscreen,
                crash_reports,
                crash_reports_available,
                discord_rpc,
                master_url,
                locale,
                server_settings,
                console,
                system_memory_mb,
            } => {
                if let Some(loc) = i18n::Locale::from_code(&locale) {
                    self.locale = loc;
                    i18n::set_locale(loc);
                }
                self.config = UiConfig {
                    memory_min_mb,
                    memory_max_mb,
                    jvm_flags,
                    show_console_on_launch,
                    fullscreen,
                    crash_reports,
                    crash_reports_available,
                    discord_rpc,
                    master_url,
                    console,
                    system_memory_mb,
                };
                self.server_settings = server_settings.into_iter().collect();
                self.load_preset_renders(cx);
            }
            MessageToFrontend::LocaleCatalog { code, ftl } => {
                // The master's catalog overrides the built-in one; a broken one
                // is ignored.
                if let Some(loc) = i18n::Locale::from_code(&code) {
                    if loc == self.locale && !i18n::install_catalog(loc, &ftl) {
                        tracing::warn!("translation catalog from the master did not parse");
                    }
                }
            }

            MessageToFrontend::ConnectionState { online } => {
                let was_lost = self.connection_lost;
                self.online = online;
                self.connection_lost = !online;
                // Images that failed while offline are worth one more try:
                // started without a network, the launcher otherwise showed no
                // icons or backgrounds for the whole session.
                if online && was_lost {
                    self.image_failed.clear();
                    self.optional_mod_icons_failed.clear();
                }
            }
            MessageToFrontend::OpenOrFocusMainWindow => {
                // A second launch hands over to this one. Deferred: raising the
                // window updates it, and this runs inside an update of its view.
                if let Some(handle) = self.main_window {
                    cx.defer(move |cx| {
                        let _ = handle.update(cx, |_, window, _| window.activate_window());
                        cx.activate(true);
                    });
                }
            }
            MessageToFrontend::CloseModal => {
                self.logging_in = false;
            }
            MessageToFrontend::Quit => {
                cx.quit();
            }
            MessageToFrontend::LauncherUpdateAvailable { version } => {
                self.update_available = Some(version);
            }
            MessageToFrontend::AddNotification { key, args, level } => {
                self.notify_toast(translate_notification(&key, &args), level, cx);
            }
            MessageToFrontend::ImpersonatePrompt {
                grant_id,
                target_username,
                reason,
                expires_in_secs,
                ..
            } => {
                self.impersonate_prompt = Some(ImpersonatePrompt {
                    grant_id,
                    target_username,
                    reason,
                    expires_at: std::time::Instant::now()
                        + std::time::Duration::from_secs(expires_in_secs.max(0) as u64),
                });
                self.tick_impersonate_prompt(grant_id, cx);
            }
            MessageToFrontend::LogRequestPrompt {
                request_id,
                actor_username,
                reason,
                forced,
                preview,
                files,
            } => {
                self.log_request_preview_open = false;
                self.log_request_prompt = Some(LogRequestPrompt {
                    request_id,
                    actor_username,
                    reason,
                    forced,
                    preview,
                    files,
                });
            }
            MessageToFrontend::RemoteActionPrompt {
                action,
                server_id,
                actor_username,
            } => {
                self.remote_action_prompt = Some(RemoteActionPrompt {
                    action,
                    server_id,
                    actor_username,
                });
            }
            MessageToFrontend::ImpersonationChanged { as_username } => {
                self.impersonate_prompt = None;
                self.impersonating_as = as_username;
                // Another account now: what was loaded belongs to the last one.
                self.clear_account_data();
            }
            MessageToFrontend::PermissionsUpdated { user } => {
                self.user = Some(user);
                self.load_user_skin(cx);
            }
            _ => {}
        }
    }
}
