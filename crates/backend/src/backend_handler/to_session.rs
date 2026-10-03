//! Window requests about sign-in and sign-out, answers to staff, the support bundle, updates and quitting.

use super::*;

impl BackendState {
    /// Only ever gets the variants `handle_to_backend` routes here.
    pub(super) async fn on_session_request(&mut self, msg: MessageToBackend) {
        match msg {
            MessageToBackend::StartWebLogin { modal_action } => {
                let master_url = self.ctx.config.get().master_url;
                let internal = self.ctx.internal.clone();
                let modal = modal_action.clone();
                modal.set_stage("Waiting for sign in on the website...");
                tokio::spawn(async move {
                    let cancelled = {
                        let m = modal.clone();
                        move || m.is_cancelled()
                    };
                    match web_login::login(&master_url, cancelled).await {
                        Ok(res) => {
                            let _ = internal.send(InternalEvent::LoginCompleted {
                                auth: res.auth,
                                user: res.user,
                            });
                        }
                        Err(e) => {
                            let kind = if e.downcast_ref::<web_login::LoginCancelled>().is_some() {
                                LoginErrorKind::Cancelled
                            } else {
                                LoginErrorKind::Network(format!("{e:#}"))
                            };
                            let _ = internal.send(InternalEvent::LoginFailed { kind });
                        }
                    }
                });
            }
            MessageToBackend::Logout => self.sign_out(),
            MessageToBackend::RemoteActionAnswer {
                action,
                server_id,
                accepted,
            } => {
                if accepted {
                    self.perform_remote_action(action, server_id);
                }
            }
            MessageToBackend::LogRequestAnswer {
                request_id,
                accepted,
            } => {
                self.answer_log_request(request_id, accepted);
            }
            MessageToBackend::ImpersonateAnswer { grant_id, accepted } => {
                self.answer_impersonate(grant_id, accepted);
            }
            MessageToBackend::ImpersonateExit => {
                self.exit_impersonation();
            }
            MessageToBackend::SendSupportBundle { server_id } => {
                self.send_support_bundle(server_id);
            }
            MessageToBackend::InstallUpdate {
                version,
                modal_action,
            } => {
                let ctx = self.ctx.clone();
                let modal = modal_action.clone();
                modal.set_stage("Downloading update...");
                tokio::spawn(async move {
                    let result = crate::updater::install_update(
                        &ctx.http,
                        &ctx.dirs,
                        &version,
                        |done, total| {
                            modal.set_progress(done, total);
                        },
                    )
                    .await;
                    match result {
                        Ok(exe) => {
                            modal.finish();
                            let _ = ctx.internal.send(InternalEvent::RestartInto(exe));
                        }
                        Err(e) => {
                            modal.fail(format!("{e:#}"));
                            ctx.send(MessageToFrontend::AddNotification {
                                key: "notif-update-failed".into(),
                                args: [("reason".to_string(), format!("{e:#}"))].into(),
                                level: schema::NotifLevel::Error,
                            });
                        }
                    }
                });
            }
            MessageToBackend::SetLocale { code } => {
                self.ctx.config.update(|c| c.locale = code.clone());
                crate::translations::refresh(&self.ctx, code);
            }
            MessageToBackend::FocusWindow => {
                self.ctx.send(MessageToFrontend::OpenOrFocusMainWindow);
            }
            MessageToBackend::Quit => {
                let running: Vec<_> = self.ctx.running.lock().keys().copied().collect();
                for id in running {
                    if let Some(g) = self.ctx.running.lock().get(&id) {
                        let _ = g.kill.send(());
                    }
                }
            }
            _ => {}
        }
    }
}
