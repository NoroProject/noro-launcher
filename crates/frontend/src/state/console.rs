//! The game console window and the log lines it shows.

use super::*;

const CONSOLE_WINDOW_SIZE: (f32, f32) = (800., 500.);
const CONSOLE_WINDOW_MIN_SIZE: (f32, f32) = (720., 440.);

pub struct ConsoleWindow {
    pub server_id: Uuid,
    pub buffer: crate::console_model::ConsoleBuffer,
    pub list_state: ListState,
    pub status_message: String,
    pub copy_success: bool,
    /// The first click on "clear" asks; a second within a few seconds clears.
    pub clear_armed: bool,
}

impl ConsoleWindow {
    fn new(server_id: Uuid, lines: impl IntoIterator<Item = LogEntry>) -> Self {
        let buffer = crate::console_model::ConsoleBuffer::from_lines(lines, Default::default());
        let list_state = ListState::new(buffer.visible_len(), ListAlignment::Top, px(100.));
        // Follows new lines while the reader is at the bottom; scrolling up
        // stops it, and scrolling back down picks it up again. Recreating the
        // list for every batch used to throw the reader back to the bottom.
        list_state.set_follow_mode(gpui::FollowMode::Tail);
        Self {
            server_id,
            buffer,
            list_state,
            status_message: String::new(),
            copy_success: false,
            clear_armed: false,
        }
    }

    /// New lines go in as a splice at the end (and one at the front for what
    /// fell off), so the reader's position holds.
    pub fn push(&mut self, lines: Vec<LogEntry>) {
        let before = self.buffer.visible_len();
        let change = self.buffer.append(lines);
        if change.removed_front > 0 {
            self.list_state.splice(0..change.removed_front, 0);
        }
        let end = before - change.removed_front;
        if change.added_back > 0 {
            self.list_state.splice(end..end, change.added_back);
        }
    }

    pub fn set_filters(&mut self, filters: crate::console_model::Filters) {
        self.buffer.set_filters(filters);
        self.list_state.reset(self.buffer.visible_len());
        self.list_state.set_follow_mode(gpui::FollowMode::Tail);
    }

    /// The console follows the server the player is looking at.
    pub fn show_server(&mut self, server_id: Uuid, lines: impl IntoIterator<Item = LogEntry>) {
        let filters = self.buffer.filters().clone();
        self.server_id = server_id;
        self.buffer = crate::console_model::ConsoleBuffer::from_lines(lines, filters);
        self.list_state.reset(self.buffer.visible_len());
        self.list_state.set_follow_mode(gpui::FollowMode::Tail);
    }

    pub fn follow(&mut self) {
        self.list_state.set_follow_mode(gpui::FollowMode::Tail);
    }
}

impl gpui::Render for ConsoleWindow {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        use crate::pages::game_console;
        game_console::console_window_body(self, cx)
    }
}

impl LauncherUI {
    pub fn toggle_console(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_server_id() {
            self.open_console(id, cx);
        }
    }

    pub fn open_console(&mut self, server_id: Uuid, cx: &mut Context<Self>) {
        if let Some(handle) = &self.console_window {
            let lines: Vec<LogEntry> = self
                .logs
                .get(&server_id)
                .map(|l| l.iter().cloned().collect())
                .unwrap_or_default();
            let _ = handle.update(cx, |view, window, cx| {
                if view.server_id != server_id {
                    view.show_server(server_id, lines);
                }
                window.activate_window();
                cx.notify();
            });
            return;
        }

        let bounds = gpui::Bounds::centered(
            None,
            gpui::size(px(CONSOLE_WINDOW_SIZE.0), px(CONSOLE_WINDOW_SIZE.1)),
            cx,
        );
        let logs: Vec<LogEntry> = self
            .logs
            .get(&server_id)
            .map(|l| l.iter().cloned().collect())
            .unwrap_or_default();
        let handle = cx.open_window(
            gpui::WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                window_min_size: Some(gpui::size(
                    px(CONSOLE_WINDOW_MIN_SIZE.0),
                    px(CONSOLE_WINDOW_MIN_SIZE.1),
                )),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some(i18n::t("console-title").into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |_, cx| {
                cx.new(|cx| {
                    cx.on_release(|_: &mut ConsoleWindow, cx| {
                        if let Some(ui) = cx.try_global::<GlobalLauncherUI>() {
                            let ui = ui.0.clone();
                            ui.update(cx, |this_ui, cx| {
                                this_ui.console_window = None;
                                cx.notify();
                            });
                        }
                    })
                    .detach();
                    ConsoleWindow::new(server_id, logs)
                })
            },
        );

        match handle {
            Ok(h) => self.console_window = Some(h),
            Err(e) => tracing::warn!(error = %e, "console window did not open"),
        }
    }

    /// Into the buffer, and to the console window if it shows this server.
    pub(super) fn on_game_log(
        &mut self,
        server_id: Uuid,
        lines: Vec<LogEntry>,
        cx: &mut Context<Self>,
    ) {
        let logs = self.logs.entry(server_id).or_default();
        logs.extend(lines.iter().cloned());
        while logs.len() > MAX_LOG_LINES {
            logs.pop_front();
        }

        if let Some(handle) = &self.console_window {
            let _ = handle.update(cx, |view, _, cx| {
                if view.server_id == server_id {
                    view.push(lines);
                    cx.notify();
                }
            });
        }
    }
}
