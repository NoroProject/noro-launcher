use crate::console_toolbar::toolbar;
use crate::state::{ConsoleWindow, LogEntry};
use crate::theme::*;
use bridge::GameLogLevel;
use chrono::{DateTime, Local, TimeZone};
use gpui::{div, list, prelude::*, px, rgb, AnyElement, FontWeight};

pub fn console_window_body(
    view: &ConsoleWindow,
    cx: &mut gpui::Context<ConsoleWindow>,
) -> AnyElement {
    // Rows are read from the view when the list asks for them: copying the
    // filtered log for every frame cost more than drawing it.
    let entity = cx.entity();
    div()
        .size_full()
        .bg(rgb(BG_WINDOW))
        .flex()
        .flex_col()
        .child(toolbar(view, cx))
        .child(
            div().flex_1().bg(rgb(0x050a12)).p(px(8.)).child(
                list(view.list_state.clone(), move |i, _, cx| {
                    match entity.read(cx).buffer.visible(i) {
                        Some(entry) => log_row(entry).into_any_element(),
                        None => div().into_any_element(),
                    }
                })
                .size_full(),
            ),
        )
        .into_any_element()
}

pub fn log_row(entry: &LogEntry) -> AnyElement {
    let color = match entry.level {
        GameLogLevel::Error => ERROR,
        GameLogLevel::Warn => WARNING,
        GameLogLevel::Info => TEXT_PRIMARY,
    };

    let time = Local
        .timestamp_millis_opt(entry.timestamp)
        .single()
        .map(|t: DateTime<Local>| t.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "00:00:00".to_string());

    let level_label = match entry.level {
        GameLogLevel::Error => "ERR",
        GameLogLevel::Warn => "WRN",
        GameLogLevel::Info => "INF",
    };

    div()
        .w_full()
        .flex()
        .gap(px(12.))
        .py(px(2.))
        .child(
            div()
                .w(px(64.))
                .flex_shrink_0()
                .text_color(rgb(TEXT_MUTED))
                .font_family("Courier New")
                .text_size(px(12.))
                .child(time),
        )
        .child(
            div()
                .w(px(32.))
                .flex_shrink_0()
                .text_color(rgb(color))
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .font_weight(FontWeight::BOLD)
                .child(level_label),
        )
        .child(
            div()
                .flex_1()
                .text_color(rgb(color))
                .font_family("Courier New")
                .text_size(px(12.))
                .overflow_x_hidden()
                .child(entry.text.clone()),
        )
        .into_any_element()
}
