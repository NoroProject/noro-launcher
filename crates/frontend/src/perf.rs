// Over 150 lines: the counters and the overlay that reads them. Apart, the
// numbers have nowhere to be seen and the overlay has nothing to show.
//! Frame counters, and the overlay that shows them.
//!
//! Switched on with `NORO_PERF=1`. Off, it costs one `Instant::now()` per frame
//! and nothing else.
//!
//! It answers the three questions that otherwise turn into guesswork:
//!
//! * **frame time** — how long a frame costs. The number to watch: GPUI only
//!   draws when something changed, so a still window legitimately sits at a
//!   couple of frames a second and that is not a problem. The overlay says
//!   `idle` there rather than a frightening `2 fps`.
//! * **redraws in the last second while nobody touches the window.** GPUI only
//!   draws after `cx.notify()`, so a screen sitting at 60 is being woken by
//!   something on a loop — the usual cause is work done in `render`, which runs
//!   per frame.
//! * **msgs/s** — requests leaving for the master. Anything above zero on an
//!   idle screen is a request made from `render`: the answer arrives, notifies,
//!   draws a frame, which asks again. That loop reads as "everything lags",
//!   because it is sixty requests a second, each with a redraw behind it.
//!
//! A counter of images that failed to load is there for the same reason: a
//! broken icon used to be retried every frame, and that is invisible until
//! something counts it.

use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight};
use std::time::Instant;

/// Ring of recent frame times, plus what the counters looked like a second ago.
pub struct Perf {
    enabled: bool,
    frames: Vec<f32>,
    last_frame_at: Option<Instant>,
    window_started: Instant,
    frames_in_window: u32,
    /// Frames per second and messages per second, as of the last full second.
    pub fps: u32,
    pub msgs_per_sec: u64,
    msgs_at_window_start: u64,
}

impl Default for Perf {
    fn default() -> Self {
        Self {
            // The launcher ships with this off: it is a development tool, and a
            // counter in the corner of somebody's game launcher is noise.
            enabled: std::env::var("NORO_PERF").is_ok_and(|v| v != "0"),
            frames: Vec::with_capacity(120),
            last_frame_at: None,
            window_started: Instant::now(),
            frames_in_window: 0,
            fps: 0,
            msgs_per_sec: 0,
            msgs_at_window_start: 0,
        }
    }
}

impl Perf {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Called once at the top of every render.
    pub fn frame(&mut self, sent_total: u64) {
        if !self.enabled {
            return;
        }
        let now = Instant::now();
        if let Some(last) = self.last_frame_at {
            let ms = now.duration_since(last).as_secs_f32() * 1000.0;
            if self.frames.len() == 120 {
                self.frames.remove(0);
            }
            self.frames.push(ms);
        }
        self.last_frame_at = Some(now);
        self.frames_in_window += 1;

        // A whole second of frames, then the numbers move. Recomputing per frame
        // would make them jitter too much to read.
        if now.duration_since(self.window_started).as_secs_f32() >= 1.0 {
            self.fps = self.frames_in_window;
            self.msgs_per_sec = sent_total.saturating_sub(self.msgs_at_window_start);
            self.msgs_at_window_start = sent_total;
            self.frames_in_window = 0;
            self.window_started = now;
        }
    }

    /// Average and worst frame over the ring, in milliseconds.
    fn timings(&self) -> (f32, f32) {
        if self.frames.is_empty() {
            return (0.0, 0.0);
        }
        let sum: f32 = self.frames.iter().sum();
        let worst = self.frames.iter().cloned().fold(0.0_f32, f32::max);
        (sum / self.frames.len() as f32, worst)
    }
}

/// The overlay itself. `None` unless `NORO_PERF` is set.
pub fn overlay(ui: &LauncherUI) -> Option<AnyElement> {
    if !ui.perf.enabled() {
        return None;
    }
    let (avg, worst) = ui.perf.timings();
    // Below ten frames a second the window is simply idle: GPUI draws on events,
    // and at rest there are no frames at all. A "2 fps" reading there alarms for
    // nothing; the frame time is what to look at.
    let idle = ui.perf.fps < 10;
    // What is alarming is a screen that keeps redrawing on its own.
    let busy = ui.perf.msgs_per_sec > 2 || (ui.perf.fps > 50 && avg > 16.0);

    Some(
        div()
            .absolute()
            .top(px(8.))
            .right(px(8.))
            .px(px(10.))
            .py(px(6.))
            .rounded(px(R_SM))
            .bg(rgba(0x000000cc))
            .border_1()
            .border_color(rgb(if busy { WARNING } else { BORDER }))
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(line(
                if idle {
                    format!("idle · {avg:.1} ms")
                } else {
                    format!("{} fps · {avg:.1} ms", ui.perf.fps)
                },
                if busy {
                    WARNING
                } else if idle {
                    TEXT_MUTED
                } else {
                    SUCCESS
                },
            ))
            .child(line(format!("worst {worst:.1} ms"), TEXT_MUTED))
            .child(line(
                format!("{} msg/s", ui.perf.msgs_per_sec),
                if ui.perf.msgs_per_sec > 2 {
                    ERROR
                } else {
                    TEXT_MUTED
                },
            ))
            .child(line(
                format!(
                    "img {} ok · {} fail",
                    ui.optional_mod_icons.len(),
                    ui.failed_image_count()
                ),
                TEXT_MUTED,
            ))
            .into_any_element(),
    )
}

fn line(text: String, colour: u32) -> AnyElement {
    div()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(10.))
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(colour))
        .child(text)
        .into_any_element()
}
