//! Window requests about skins, capes, notifications, punishments, rules, tickets and messages.

use super::*;

impl BackendState {
    /// Only ever gets the variants `handle_to_backend` routes here.
    pub(super) async fn on_account_request(&mut self, msg: MessageToBackend) {
        match msg {
            MessageToBackend::SetSkinModel { slim } => {
                let Some(api) = crate::master_api::MasterApi::for_session(&self.ctx) else {
                    return;
                };
                let ctx = self.ctx.clone();
                tokio::spawn(async move {
                    match api.set_skin_model(slim).await {
                        Ok(profile) => {
                            let _ = ctx
                                .internal
                                .send(InternalEvent::ProfileUpdated { user: profile });
                        }
                        Err(e) => ctx.send(MessageToFrontend::AddNotification {
                            key: "notif-skin-model-failed".into(),
                            args: [("reason".to_string(), format!("{e:#}"))].into(),
                            level: schema::NotifLevel::Error,
                        }),
                    }
                });
            }
            MessageToBackend::UploadSkin { bytes } => {
                let Some(api) = crate::master_api::MasterApi::for_session(&self.ctx) else {
                    self.notify("notif-sign-in-to-upload", schema::NotifLevel::Error);
                    self.ctx.send(MessageToFrontend::SkinUploadFailed);
                    return;
                };
                let ctx = self.ctx.clone();
                tokio::spawn(async move {
                    match api.upload_skin(bytes).await {
                        Ok(profile) => {
                            let _ = ctx
                                .internal
                                .send(InternalEvent::ProfileUpdated { user: profile });
                        }
                        Err(e) => {
                            ctx.send(MessageToFrontend::AddNotification {
                                key: "notif-skin-upload-failed".into(),
                                args: [("reason".to_string(), format!("{e:#}"))].into(),
                                level: schema::NotifLevel::Error,
                            });
                            ctx.send(MessageToFrontend::SkinUploadFailed);
                        }
                    }
                });
            }
            MessageToBackend::RequestCapesList => {
                let Some(api) = crate::master_api::MasterApi::for_session(&self.ctx) else {
                    return;
                };
                let ctx = self.ctx.clone();
                tokio::spawn(async move {
                    match api.capes().await {
                        Ok(capes) => ctx.send(MessageToFrontend::CapesList { capes }),
                        Err(e) => tracing::warn!(error = %format!("{e:#}"), "capes did not load"),
                    }
                });
            }
            MessageToBackend::RequestSkinPresetsList => self.request_skin_presets(),
            MessageToBackend::SelectCape { cape_id } => {
                let Some(api) = crate::master_api::MasterApi::for_session(&self.ctx) else {
                    return;
                };
                let ctx = self.ctx.clone();
                tokio::spawn(async move {
                    match api.select_cape(cape_id).await {
                        Ok(profile) => {
                            let _ = ctx.internal.send(InternalEvent::ProfileUpdated {
                                user: profile.clone(),
                            });
                            ctx.send(MessageToFrontend::PermissionsUpdated { user: profile });
                        }
                        Err(e) => ctx.send(MessageToFrontend::AddNotification {
                            key: "notif-cape-update-failed".into(),
                            args: [("reason".to_string(), format!("{e:#}"))].into(),
                            level: schema::NotifLevel::Error,
                        }),
                    }
                });
            }
            MessageToBackend::RequestNotifications {
                offset,
                unread_only,
            } => crate::notifications::request(&self.ctx, offset, unread_only),
            MessageToBackend::MarkNotificationRead { id } => {
                crate::notifications::mark_read(&self.ctx, id)
            }
            MessageToBackend::MarkAllNotificationsRead => {
                crate::notifications::mark_all_read(&self.ctx)
            }
            MessageToBackend::RequestPunishments => crate::account::punishments(&self.ctx),
            MessageToBackend::RequestRules => crate::account::rules(&self.ctx),
            MessageToBackend::RequestTickets => crate::account::tickets(&self.ctx),
            MessageToBackend::RequestTicket { id } => crate::account::ticket(&self.ctx, id),
            MessageToBackend::OpenTicket { subject, content } => {
                crate::account::open_ticket(&self.ctx, subject, content)
            }
            MessageToBackend::ReplyTicket { id, content } => {
                crate::account::reply_ticket(&self.ctx, id, content)
            }
            MessageToBackend::RequestDmThreads => crate::account::dm_threads(&self.ctx),
            MessageToBackend::RequestDmThread { peer } => {
                crate::account::dm_thread(&self.ctx, peer)
            }
            MessageToBackend::SendDm { peer, body } => {
                crate::account::send_dm(&self.ctx, peer, body)
            }
            _ => {}
        }
    }
}
