//! Requests from staff: remote actions, log requests, impersonation, and the
//! player's own support bundle.

use super::*;

impl BackendState {
    /// Anything that erases files or interrupts the player asks them first.
    pub(super) fn run_remote_action(
        &mut self,
        action: schema::RemoteAction,
        server_id: Option<Uuid>,
        actor_username: String,
    ) {
        if action.needs_confirmation() {
            self.ctx.send(MessageToFrontend::RemoteActionPrompt {
                action,
                server_id,
                actor_username,
            });
            return;
        }
        self.perform_remote_action(action, server_id);
    }

    /// The run itself: after confirmation, or straight away when none is needed.
    pub fn perform_remote_action(&mut self, action: schema::RemoteAction, server_id: Option<Uuid>) {
        if action == schema::RemoteAction::KillGame {
            let running: Vec<_> = self.ctx.running.lock().keys().copied().collect();
            let mut killed = 0;
            for id in running {
                if server_id.is_none() || server_id == Some(id) {
                    if let Some(g) = self.ctx.running.lock().get(&id) {
                        let _ = g.kill.send(());
                        killed += 1;
                    }
                }
            }
            self.ctx.send(MessageToFrontend::AddNotification {
                key: "notif-remote-action-done".into(),
                args: [(
                    "detail".to_string(),
                    format!("game process stopped ({killed})"),
                )]
                .into(),
                level: schema::NotifLevel::Info,
            });
            return;
        }

        if action == schema::RemoteAction::RestartLauncher {
            let ctx = self.ctx.clone();
            tokio::spawn(async move {
                ctx.send(MessageToFrontend::AddNotification {
                    key: "notif-remote-action-done".into(),
                    args: [("detail".to_string(), "the launcher is restarting...".into())].into(),
                    level: schema::NotifLevel::Info,
                });
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if let Ok(current_exe) = std::env::current_exe() {
                    crate::updater::restart(&current_exe);
                }
            });
            return;
        }

        // Deleting mods or assets under a running JVM breaks the session and
        // the files only half go; it waits until the game is closed.
        let destructive = matches!(
            action,
            schema::RemoteAction::ReinstallBuild | schema::RemoteAction::ClearAssetCache
        );
        let busy = {
            let running = self.ctx.running.lock();
            match server_id {
                Some(id) => running.contains_key(&id),
                None => !running.is_empty(),
            }
        };
        if destructive && busy {
            self.notify("notif-remote-action-busy", schema::NotifLevel::Warning);
            return;
        }

        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            match crate::remote_actions::run(&ctx.dirs, action, server_id).await {
                Ok(outcome) => {
                    tracing::info!(action = action.as_str(), "{}", outcome.message);
                    ctx.send(MessageToFrontend::AddNotification {
                        key: "notif-remote-action-done".into(),
                        args: [("detail".to_string(), outcome.message)].into(),
                        level: schema::NotifLevel::Info,
                    });
                }
                Err(e) => {
                    tracing::warn!(error = %format!("{e:#}"), action = action.as_str(), "action failed")
                }
            }
        });
    }

    /// Collect the bundle and show the player exactly what would leave their
    /// machine. Forced mode sends it anyway, but still shows the modal.
    pub(super) fn prepare_log_request(
        &mut self,
        request_id: Uuid,
        actor_username: String,
        reason: String,
        forced: bool,
        target_server_id: Option<Uuid>,
    ) {
        let server_id = target_server_id.or_else(|| self.fallback_log_server());
        let instance_dir = match server_id {
            Some(ref id) => self.ctx.dirs.instance(id),
            None => self.ctx.dirs.root.clone(),
        };
        let ctx = self.ctx.clone();
        let token = self.access_token.clone();
        let master = self.ctx.config.get().master_url;

        tokio::spawn(async move {
            let bundle = crate::support::collect(&instance_dir, None, &[]).await;
            let files = bundle
                .files
                .iter()
                .map(|f| (f.name.clone(), f.original_bytes))
                .collect();

            ctx.send(MessageToFrontend::LogRequestPrompt {
                request_id,
                actor_username,
                reason,
                forced,
                preview: bundle.preview(),
                files,
            });

            // Forced mode doesn't wait for the answer — the prompt above is a
            // notice, not a question.
            if forced {
                if let Some(token) = token {
                    let _ = crate::support::send_for_request(
                        &ctx.http,
                        &master,
                        &token,
                        &instance_dir,
                        server_id,
                        request_id,
                    )
                    .await;
                }
            }
        });
    }

    pub(super) fn answer_log_request(&mut self, request_id: Uuid, accepted: bool) {
        self.ctx.ws.send(ClientWsMsg::LogRequestResponse {
            request_id,
            accepted,
        });
        if !accepted {
            return;
        }
        let Some(token) = self.access_token.clone() else {
            return;
        };
        let server_id = self.fallback_log_server();
        let instance_dir = match server_id {
            Some(ref id) => self.ctx.dirs.instance(id),
            None => self.ctx.dirs.root.clone(),
        };
        let ctx = self.ctx.clone();
        let master = self.ctx.config.get().master_url;
        tokio::spawn(async move {
            match crate::support::send_for_request(
                &ctx.http,
                &master,
                &token,
                &instance_dir,
                server_id,
                request_id,
            )
            .await
            {
                Ok(_) => ctx.send(MessageToFrontend::AddNotification {
                    key: "notif-support-sent".into(),
                    args: std::collections::BTreeMap::new(),
                    level: schema::NotifLevel::Info,
                }),
                Err(e) => tracing::warn!(error = %format!("{e:#}"), "requested logs were not sent"),
            }
        });
    }

    /// A refusal has to go back too: the master waits for an answer, and
    /// silence leaves the admin's page polling until the grant expires.
    pub(super) fn answer_impersonate(&mut self, grant_id: Uuid, accepted: bool) {
        self.ctx
            .ws
            .send(ClientWsMsg::ImpersonateResponse { grant_id, accepted });
        if !accepted {
            return;
        }

        let Some(token) = self.access_token.clone() else {
            return;
        };
        // Keep our own token before the swap: leaving the other account means
        // going back to it, not signing in again.
        self.own_token = Some(token.clone());
        let ctx = self.ctx.clone();
        let master = self.ctx.config.get().master_url;
        let internal = self.ctx.internal.clone();
        tokio::spawn(async move {
            match crate::impersonation::claim(&ctx.http, &master, &token, grant_id).await {
                Ok(claimed) => {
                    let _ = internal.send(crate::backend::InternalEvent::ImpersonationStarted {
                        access_token: claimed.access_token,
                        username: claimed.username,
                    });
                }
                Err(e) => {
                    tracing::warn!(error = %format!("{e:#}"), "could not enter the player's account");
                    ctx.send(MessageToFrontend::AddNotification {
                        key: "notif-impersonate-failed".into(),
                        args: [("reason".to_string(), format!("{e:#}"))].into(),
                        level: schema::NotifLevel::Error,
                    });
                }
            }
        });
    }

    pub(crate) fn exit_impersonation(&mut self) {
        let Some(own) = self.own_token.take() else {
            return;
        };
        self.access_token = Some(own.clone());
        self.ctx.ws.set_token(Some(own));
        self.ctx
            .send(MessageToFrontend::ImpersonationChanged { as_username: None });
    }

    /// "Report a problem": collect the logs and send them to the master.
    ///
    /// Runs in the background, since collecting reads files off disk; the
    /// result comes back as a notification.
    pub(super) fn send_support_bundle(&self, server_id: Option<Uuid>) {
        let Some(token) = self.access_token.clone() else {
            self.notify("notif-sign-in-first", schema::NotifLevel::Error);
            return;
        };
        // No server means no logs: the game writes them into the instance
        // directory.
        let Some(server_id) = server_id.or_else(|| self.fallback_log_server()) else {
            self.notify("notif-support-nothing-to-send", schema::NotifLevel::Warning);
            return;
        };

        let ctx = self.ctx.clone();
        let instance_dir = self.ctx.dirs.instance(&server_id);
        let master = self.ctx.config.get().master_url;
        tokio::spawn(async move {
            match crate::support::send(
                &ctx.http,
                &master,
                &token,
                &instance_dir,
                Some(server_id),
                "",
            )
            .await
            {
                Ok(id) => {
                    tracing::info!(%id, "support bundle sent");
                    ctx.send(MessageToFrontend::AddNotification {
                        key: "notif-support-sent".into(),
                        args: std::collections::BTreeMap::new(),
                        level: schema::NotifLevel::Info,
                    });
                }
                Err(e) => {
                    tracing::warn!(error = %format!("{e:#}"), "support bundle not sent");
                    ctx.send(MessageToFrontend::AddNotification {
                        key: "notif-support-failed".into(),
                        args: [("reason".to_string(), format!("{e:#}"))].into(),
                        level: schema::NotifLevel::Error,
                    });
                }
            }
        });
    }
}
