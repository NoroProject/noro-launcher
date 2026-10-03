// Over 150 lines: the log the window holds, which of it is shown, and how the
// list follows it — three views of one buffer that only stay in step together.
//! The game console's window.
//!
//! Lines are drawn straight from here, by index. The list used to get a copy of
//! every shown line on each render, and a modded client renders the console as
//! often as it prints — thousands of times on its way to the menu.

use crate::state::{GlobalLauncherUI, LogEntry};
use bridge::{BackendHandle, ConsoleSettings, GameLogLevel, MessageToBackend};
use gpui::{px, Context, FocusHandle, FollowMode, ListAlignment, ListState};
use uuid::Uuid;

/// Lines kept per server. GTNH prints a few thousand on its way to the menu,
/// and the reason it didn't start is usually near the top.
pub const MAX_LOG_LINES: usize = 5000;

pub struct ConsoleWindow {
    pub server_id: Uuid,
    pub server_name: String,
    pub logs: Vec<LogEntry>,
    /// Indices into `logs` of the lines that pass the level filter and search.
    pub visible: Vec<usize>,
    pub list_state: ListState,
    pub settings: ConsoleSettings,
    pub search: String,
    pub search_focus: Option<FocusHandle>,
    pub settings_open: bool,
    /// The copy button says so for a moment.
    pub copied: bool,
    backend: BackendHandle,
}

impl ConsoleWindow {
    pub fn new(
        server_id: Uuid,
        server_name: String,
        logs: Vec<LogEntry>,
        settings: ConsoleSettings,
        backend: BackendHandle,
    ) -> Self {
        let list_state = ListState::new(0, ListAlignment::Bottom, px(200.));
        // Follows the tail and lets go when the reader scrolls up; comes back
        // on its own once they scroll down to the end again.
        list_state.set_follow_mode(FollowMode::Tail);
        let mut view = Self {
            server_id,
            server_name,
            logs,
            visible: Vec::new(),
            list_state,
            settings,
            search: String::new(),
            search_focus: None,
            settings_open: false,
            copied: false,
            backend,
        };
        view.refilter();
        view
    }

    fn query(&self) -> String {
        self.search.trim().to_lowercase()
    }

    fn passes(&self, entry: &LogEntry, query: &str) -> bool {
        let level = match entry.level {
            GameLogLevel::Info => self.settings.show_info,
            GameLogLevel::Warn => self.settings.show_warn,
            GameLogLevel::Error => self.settings.show_error,
        };
        level && (query.is_empty() || entry.text.to_lowercase().contains(query))
    }

    /// Recount what is shown: the filter or the search changed.
    pub fn refilter(&mut self) {
        let query = self.query();
        self.visible = (0..self.logs.len())
            .filter(|&i| self.passes(&self.logs[i], &query))
            .collect();
        self.list_state.reset(self.visible.len());
    }

    /// New lines from the game. The list hears only what changed: a reset
    /// would throw someone reading further up back to the end on every line.
    pub fn append(&mut self, lines: Vec<LogEntry>) {
        let query = self.query();
        let shown_before = self.visible.len();
        let start = self.logs.len();
        self.logs.extend(lines);
        for i in start..self.logs.len() {
            if self.passes(&self.logs[i], &query) {
                self.visible.push(i);
            }
        }

        let overflow = self.logs.len().saturating_sub(MAX_LOG_LINES);
        let mut dropped = 0;
        if overflow > 0 {
            self.logs.drain(..overflow);
            dropped = self.visible.partition_point(|&i| i < overflow);
            self.visible.drain(..dropped);
            for i in &mut self.visible {
                *i -= overflow;
            }
        }

        if dropped > shown_before {
            // A batch bigger than the buffer pushed out lines it brought itself.
            self.list_state.reset(self.visible.len());
            return;
        }
        self.list_state.splice(0..dropped, 0);
        let kept = shown_before - dropped;
        self.list_state
            .splice(kept..kept, self.visible.len() - kept);
    }

    pub fn clear(&mut self) {
        self.logs.clear();
        self.visible.clear();
        self.list_state.reset(0);
    }

    pub fn is_following(&self) -> bool {
        self.list_state.is_following_tail()
    }

    /// Back to the end, and following it again.
    pub fn follow(&mut self) {
        self.list_state.set_follow_mode(FollowMode::Tail);
    }

    /// Per level, over the whole buffer: what the filter chips show.
    pub fn counts(&self) -> [usize; 3] {
        let mut counts = [0; 3];
        for entry in &self.logs {
            counts[level_index(entry.level)] += 1;
        }
        counts
    }

    /// Every shown line in full, as the game printed it — for pasting into a
    /// report, where the cut-off prefixes are part of the evidence.
    pub fn shown_text(&self) -> String {
        self.visible
            .iter()
            .filter_map(|&i| self.logs.get(i))
            .map(|entry| entry.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn set_search(&mut self, search: String) {
        if search != self.search {
            self.search = search;
            self.refilter();
        }
    }

    /// Change how the log is shown and keep it for the next console.
    pub fn change_settings(
        &mut self,
        change: impl FnOnce(&mut ConsoleSettings),
        cx: &mut Context<Self>,
    ) {
        let before = self.settings;
        change(&mut self.settings);
        if self.settings == before {
            return;
        }
        let settings = self.settings;
        self.backend
            .send(MessageToBackend::SetConsoleSettings { settings });
        if let Some(ui) = cx.try_global::<GlobalLauncherUI>() {
            let ui = ui.0.clone();
            ui.update(cx, |ui, _| ui.config.console = settings);
        }

        let filter = |s: &ConsoleSettings| (s.show_info, s.show_warn, s.show_error);
        if filter(&before) != filter(&settings) {
            self.refilter();
        } else {
            // Columns, wrapping and size change every row's height.
            self.list_state.remeasure();
        }
    }
}

pub fn level_index(level: GameLogLevel) -> usize {
    match level {
        GameLogLevel::Info => 0,
        GameLogLevel::Warn => 1,
        GameLogLevel::Error => 2,
    }
}

impl gpui::Render for ConsoleWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        crate::pages::game_console::console_window_body(self, cx)
    }
}

#[cfg(test)]
#[path = "console_window_tests.rs"]
mod tests;
