// Over 150 lines: the player's own actions outside a server: sign-in, overlays,
// skins and answers to staff.
//! Toasts, sign-in and sign-out, closing overlays, skins, and the answers to
//! requests from staff.

use super::*;

impl LauncherUI {
    /// Show a toast and remove it on a timer.
    ///
    /// The timer sleeps in the background and wakes the window once, to remove it.
    /// Counting the time left in render would mean redrawing for all those seconds,
    /// burning frames on a fading label.
    pub fn notify_toast(&mut self, text: String, level: NotifLevel, cx: &mut Context<Self>) {
        // The same text again doesn't become a second toast: two identical
        // lines side by side look like a glitch. The one up stays longer.
        let (id, generation, lifetime) =
            if let Some(existing) = self.toasts.iter_mut().find(|t| t.text == text) {
                existing.generation += 1;
                (existing.id, existing.generation, existing.lifetime())
            } else {
                let id = self.next_toast_id;
                self.next_toast_id += 1;
                let toast = Toast {
                    id,
                    text,
                    level,
                    generation: 0,
                };
                let lifetime = toast.lifetime();
                self.toasts.push(toast);
                // More than four is a wall, not messages; the oldest goes early.
                if self.toasts.len() > 4 {
                    self.toasts.remove(0);
                }
                (id, 0, lifetime)
            };

        // The timer sleeps in the background and wakes the window once, to
        // remove the toast. Counting down in render would redraw every frame
        // for a fading line of text.
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| {
            executor.timer(lifetime).await;
            let _ = this.update(cx, |state, cx| {
                let current = state
                    .toasts
                    .iter()
                    .any(|t| t.id == id && t.generation == generation);
                if current {
                    state.dismiss_toast(id);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub fn dismiss_toast(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }

    // --- Actions from the UI ---

    /// Sign in through the website: every provider we support lives there.
    pub fn start_login(&mut self) {
        self.logging_in = true;
        self.login_error = None;
        let modal = bridge::ModalAction::new("Website sign in");
        self.login_modal = Some(modal.clone());
        self.backend.send(MessageToBackend::StartWebLogin {
            modal_action: modal,
        });
    }

    /// Everything that belongs to the signed-in account. Left in place, the
    /// next account — another player on the same computer, or an admin
    /// entering someone's account — saw the previous one's tickets, messages
    /// and punishments, and the "already loaded" flags kept them from being
    /// fetched again.
    pub(super) fn clear_account_data(&mut self) {
        self.account_requested.clear();
        self.account_loaded.clear();
        self.dm_loaded = false;
        self.punishments.clear();
        self.tickets.clear();
        self.ticket_open = None;
        self.dm_threads.clear();
        self.dm_open = None;
        self.dm_requested = false;
        self.compose.clear();
        self.notifications.clear();
        self.notifications_total = 0;
        self.unread = 0;
        self.capes.clear();
        self.cape_images.clear();
        let custom: HashSet<String> = self.custom_presets.iter().map(|p| p.id.clone()).collect();
        self.preset_images.retain(|id, _| !custom.contains(id));
        self.custom_presets.clear();
        self.personal_content.clear();
        self.suggested_mods.clear();
    }

    /// Closes whatever sits on top; false when nothing was open.
    pub fn close_top_overlay(&mut self) -> bool {
        if std::mem::take(&mut self.close_prompt) {
            return true;
        }
        if std::mem::take(&mut self.jvm_flags_open) {
            return true;
        }
        if std::mem::take(&mut self.java_picker_open) {
            return true;
        }
        if self.content_picker.take().is_some() {
            return true;
        }
        if std::mem::take(&mut self.build_picker_open) {
            return true;
        }
        if std::mem::take(&mut self.notifications_open) {
            return true;
        }
        if std::mem::take(&mut self.log_request_preview_open) {
            return true;
        }
        if self.mod_catalog_selected.take().is_some() {
            self.mod_project = None;
            return true;
        }
        false
    }

    /// Whether the window may close now. With a game running or a download in
    /// flight it may not: closing would stop them, so the window asks first.
    pub fn request_close(&mut self) -> bool {
        let busy = self.sync.values().any(|s| s.running || s.syncing);
        if busy {
            self.close_prompt = true;
        }
        !busy
    }

    /// The sign-in task polls the flag and answers with a cancelled login.
    pub fn cancel_login(&mut self) {
        if let Some(modal) = self.login_modal.take() {
            modal.cancel();
        }
        self.logging_in = false;
    }

    pub fn logout(&mut self) {
        self.backend.send(MessageToBackend::Logout);
    }

    pub fn upload_skin(&mut self, bytes: Vec<u8>) {
        if self.skin_uploading {
            return;
        }
        // Shown right away; put back if the master turns it down, or the
        // preview went on showing a skin that was never accepted.
        self.skin_before_upload = self.skin_bytes.replace(bytes.clone());
        self.skin_uploading = true;
        self.backend.send(MessageToBackend::UploadSkin { bytes });
    }

    /// Switch the skin model. The image stays as it is and only the arm width
    /// changes; the profile comes back the same way it does after an upload.
    pub fn set_skin_model(&mut self, slim: bool, _cx: &mut Context<Self>) {
        if self.skin_uploading || self.user.as_ref().is_none_or(|u| u.skin_slim == slim) {
            return;
        }
        self.backend.send(MessageToBackend::SetSkinModel { slim });
    }

    pub fn open_news(&mut self, id: Uuid, cx: &mut Context<Self>) {
        self.page = Page::NewsDetail(id);
        self.load_news_image(id, cx);
    }

    pub fn answer_log_request(&mut self, accepted: bool) {
        let Some(prompt) = self.log_request_prompt.take() else {
            return;
        };
        self.backend.send(MessageToBackend::LogRequestAnswer {
            request_id: prompt.request_id,
            accepted,
        });
    }

    pub fn answer_remote_action(&mut self, accepted: bool) {
        let Some(prompt) = self.remote_action_prompt.take() else {
            return;
        };
        self.backend.send(MessageToBackend::RemoteActionAnswer {
            action: prompt.action,
            server_id: prompt.server_id,
            accepted,
        });
    }

    /// Close the forced-collection modal; there is nothing to answer there.
    pub fn dismiss_log_request(&mut self) {
        self.log_request_prompt = None;
    }

    /// Counts the seconds down once a second while the request is up, and
    /// takes it away when it runs out: the number used to stay where it
    /// started, and an expired request could still be "allowed".
    pub(super) fn tick_impersonate_prompt(&mut self, grant_id: Uuid, cx: &mut Context<Self>) {
        let executor = cx.background_executor().clone();
        cx.spawn(async move |this, cx| loop {
            executor.timer(std::time::Duration::from_secs(1)).await;
            let still_up = this.update(cx, |ui, cx| {
                let Some(prompt) = &ui.impersonate_prompt else {
                    return false;
                };
                if prompt.grant_id != grant_id {
                    return false;
                }
                if prompt.seconds_left() <= 0 {
                    ui.impersonate_prompt = None;
                }
                cx.notify();
                ui.impersonate_prompt.is_some()
            });
            if !matches!(still_up, Ok(true)) {
                break;
            }
        })
        .detach();
    }

    pub fn answer_impersonate(&mut self, accepted: bool) {
        let Some(prompt) = self.impersonate_prompt.take() else {
            return;
        };
        self.backend.send(MessageToBackend::ImpersonateAnswer {
            grant_id: prompt.grant_id,
            accepted,
        });
    }

    pub fn exit_impersonation(&mut self) {
        self.backend.send(MessageToBackend::ImpersonateExit);
    }

    /// No server id: the backend picks the one whose manifest is already loaded.
    pub fn send_support_bundle(&mut self) {
        self.backend
            .send(MessageToBackend::SendSupportBundle { server_id: None });
    }
}
