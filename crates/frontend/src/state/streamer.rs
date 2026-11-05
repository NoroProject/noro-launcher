//! Streamer mode in the window: the switch, and the console lines that were
//! already on screen when it went on.

use super::*;
use crate::streamer_mask::{mask, MASK};

impl LauncherUI {
    pub(super) fn on_streamer_mode(&mut self, on: bool, automatic: bool, cx: &mut Context<Self>) {
        let was = self.streamer;
        self.streamer = on;
        if on && !was {
            self.mask_logs(cx);
        }
        // Nobody pressed anything, so the player hears why the window changed.
        if on && automatic {
            self.notify_toast(i18n::t("streamer-auto-on"), NotifLevel::Info, cx);
        }
    }

    /// From settings, the window bar or the shortcut. Applied here at once so
    /// the page hides what it should in the same frame, not after a round trip.
    pub fn set_streamer(&mut self, on: bool, cx: &mut Context<Self>) {
        if on && !self.streamer {
            self.mask_logs(cx);
        }
        self.streamer = on;
        self.backend.send(MessageToBackend::SetStreamerMode { on });
    }

    pub fn set_streamer_auto(&mut self, enabled: bool) {
        self.config.streamer_auto = enabled;
        self.backend
            .send(MessageToBackend::SetStreamerAuto { enabled });
    }

    /// This network's server hosts and what may stand in for each on screen.
    pub(super) fn streamer_hosts(&self) -> Vec<(String, String)> {
        let mut hosts = Vec::new();
        for server in &self.servers {
            // Where a server alias lands, it goes here as the stand-in; until
            // then the host is simply masked.
            if let Some(host) = &server.mc_host {
                hosts.push((host.clone(), MASK.to_string()));
            }
            for node in &server.game_servers {
                hosts.push((node.mc_host.clone(), MASK.to_string()));
            }
        }
        hosts
    }

    /// A line as it may be shown now.
    pub(super) fn stream_safe(&self, mut line: LogEntry, hosts: &[(String, String)]) -> LogEntry {
        if self.streamer {
            line.text = mask(&line.text, hosts).into_owned();
            line.body = mask(&line.body, hosts).into_owned();
        }
        line
    }

    /// Lines already in the buffers were printed before the mode went on.
    /// They are masked in place: turning the mode off later does not bring
    /// the addresses back, which is the safe way round.
    fn mask_logs(&mut self, cx: &mut Context<Self>) {
        let hosts = self.streamer_hosts();
        for lines in self.logs.values_mut() {
            for line in lines.iter_mut() {
                line.text = mask(&line.text, &hosts).into_owned();
                line.body = mask(&line.body, &hosts).into_owned();
            }
        }
        if let Some(handle) = &self.console_window {
            let _ = handle.update(cx, |view, _, cx| {
                view.mask_all(|text| mask(text, &hosts).into_owned());
                cx.notify();
            });
        }
    }
}
