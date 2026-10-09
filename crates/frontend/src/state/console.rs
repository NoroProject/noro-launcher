//! Opening the game console window and feeding it log lines; the window
//! itself lives in `console_window`.

use super::*;

const CONSOLE_WINDOW_SIZE: (f32, f32) = (800., 500.);
const CONSOLE_WINDOW_MIN_SIZE: (f32, f32) = (720., 440.);

pub use crate::console_window::ConsoleWindow;

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
            let server_name = self
                .server(&server_id)
                .map(|s| s.name.clone())
                .unwrap_or_default();
            let _ = handle.update(cx, |view, window, cx| {
                if view.server_id != server_id {
                    view.show_server(server_id, server_name, lines);
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
        let server_name = self
            .server(&server_id)
            .map(|s| s.name.clone())
            .unwrap_or_default();
        let settings = self.config.console;
        let backend = self.backend.clone();
        let handle = cx.open_window(
            gpui::WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                window_min_size: Some(gpui::size(
                    px(CONSOLE_WINDOW_MIN_SIZE.0),
                    px(CONSOLE_WINDOW_MIN_SIZE.1),
                )),
                // Same chrome as the launcher: transparent titlebar, traffic
                // lights parked off-screen, the bar drawn by `console_chrome`.
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some(i18n::t("console-title").into()),
                    appears_transparent: true,
                    traffic_light_position: Some(gpui::point(px(-120.), px(-120.))),
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
                    ConsoleWindow::new(server_id, server_name, logs, settings, backend)
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
                    view.append(lines);
                    cx.notify();
                }
            });
        }
    }
}
