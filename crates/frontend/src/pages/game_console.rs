//! The console's body: its bar, the toolbar, the log, and the settings over it.

use crate::console_chrome::console_chrome;
use crate::console_model::{level_label, time_label};
use crate::console_settings::settings_panel;
use crate::console_toolbar::toolbar;
use crate::state::{ConsoleWindow, LogEntry};
use crate::theme::*;
use bridge::{ConsoleSettings, GameLogLevel};
use gpui::{div, list, prelude::*, px, rgb, rgba, AnyElement, Context, FontWeight, WeakEntity};
use i18n::t;

pub fn console_window_body(
    view: &mut ConsoleWindow,
    cx: &mut Context<ConsoleWindow>,
) -> AnyElement {
    let weak = cx.entity().downgrade();
    div()
        .size_full()
        .relative()
        .bg(rgb(BG_WINDOW))
        .flex()
        .flex_col()
        .child(console_chrome(view))
        .child(toolbar(view, cx))
        .child(log_list(view, weak))
        .children(settings_panel(view, cx))
        .into_any_element()
}

fn log_list(view: &ConsoleWindow, weak: WeakEntity<ConsoleWindow>) -> AnyElement {
    if view.visible.is_empty() {
        return div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(OVERLAY))
            .text_size(px(12.))
            .text_color(rgb(TEXT_MUTED))
            .child(if view.logs.is_empty() {
                t("console-empty")
            } else {
                t("console-no-matches")
            })
            .into_any_element();
    }
    div()
        .flex_1()
        .min_h_0()
        .bg(rgb(OVERLAY))
        .py(px(4.))
        .child(
            // Rows are read from the window by index rather than handed a copy
            // of the shown lines on every render.
            list(view.list_state.clone(), move |i, _window, cx| {
                weak.upgrade()
                    .and_then(|view| {
                        let view = view.read(cx);
                        let entry = view.visible.get(i).and_then(|&ix| view.logs.get(ix))?;
                        Some(log_row(entry, &view.settings))
                    })
                    .unwrap_or_else(|| div().into_any_element())
            })
            .size_full(),
        )
        .into_any_element()
}

fn tone(level: GameLogLevel) -> u32 {
    match level {
        GameLogLevel::Info => TEXT_SECONDARY,
        GameLogLevel::Warn => WARNING,
        GameLogLevel::Error => ERROR,
    }
}

/// A stack trace hangs under the line that started it: indented, with no
/// time, level or source of its own to repeat.
pub fn log_row(entry: &LogEntry, s: &ConsoleSettings) -> AnyElement {
    let head = !entry.continuation;
    let body_color = match entry.level {
        GameLogLevel::Info if head => TEXT_PRIMARY,
        level => tone(level),
    };
    div()
        .w_full()
        .flex()
        .items_start()
        .gap(px(8.))
        .px(px(12.))
        .py(px(1.))
        .font_family("Courier New")
        .text_size(px(f32::from(s.font_size)))
        .hover(|d| d.bg(rgba(0xffffff08)))
        .when(s.show_time, |d| {
            d.child(
                div()
                    .w(px(64.))
                    .flex_none()
                    .text_color(rgb(TEXT_MUTED))
                    .child(if head {
                        time_label(entry.timestamp)
                    } else {
                        String::new()
                    }),
            )
        })
        .child(level_badge(entry.level, head))
        .when(s.show_thread && head, |d| {
            d.children(entry.thread.clone().map(|thread| {
                div()
                    .flex_none()
                    .max_w(px(140.))
                    .truncate()
                    .text_color(rgb(TEXT_MUTED))
                    .child(thread)
            }))
        })
        .when(s.show_logger && head, |d| {
            d.children(entry.logger.clone().map(|logger| {
                div()
                    .flex_none()
                    .max_w(px(180.))
                    .truncate()
                    .text_color(rgb(BLUE))
                    .child(logger)
            }))
        })
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_color(rgb(body_color))
                .when(!head, |d| d.pl(px(16.)))
                .when(!s.wrap, |d| d.truncate())
                .child(entry.body.clone()),
        )
        .into_any_element()
}

fn level_badge(level: GameLogLevel, head: bool) -> AnyElement {
    let slot = div().w(px(48.)).flex_none();
    if !head {
        return slot.into_any_element();
    }
    let tone = tone(level);
    slot.child(
        div()
            .h(px(16.))
            .px(px(4.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(R_SM))
            // Info stays quiet: a badge on every line is no badge at all.
            .when(level != GameLogLevel::Info, |d| {
                d.bg(rgba((tone << 8) | 0x22))
            })
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(9.))
            .font_weight(FontWeight::BOLD)
            .text_color(rgb(tone))
            .child(level_label(level)),
    )
    .into_any_element()
}
