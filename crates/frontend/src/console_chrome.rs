//! The console window's bar, drawn like the launcher's own: the system titlebar
//! is transparent with its buttons parked off-screen, so dragging, minimising
//! and closing live here.

use crate::components::{chrome_control, in_resize_edge};
use crate::console_window::ConsoleWindow;
use crate::icons::ic;
use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, AnyElement, FontWeight, MouseButton, MouseDownEvent,
    WindowControlArea,
};
use i18n::t;

pub fn console_chrome(view: &ConsoleWindow) -> AnyElement {
    div()
        .id("console-chrome")
        .h(px(40.))
        .w_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(8.))
        .px(px(12.))
        .bg(rgb(BG_HEADER))
        .border_b_1()
        .border_color(rgb(BORDER))
        // Not from the resize zone at the top edge: a move started there fights
        // the resize, the same as in the launcher's own bar.
        .on_mouse_down(MouseButton::Left, |event: &MouseDownEvent, window, _| {
            if !in_resize_edge(event.position, window.viewport_size()) {
                window.start_window_move();
            }
        })
        .child(ic("terminal", 14., ACCENT))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(13.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(t("console-title")),
        )
        .child(
            div()
                .min_w_0()
                .truncate()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .text_color(rgb(TEXT_MUTED))
                .child(view.server_name.clone()),
        )
        // Windows drags by its own hit test, and only this stretch is marked:
        // the whole bar would make the buttons title bar too.
        .child(
            div()
                .flex_1()
                .h_full()
                .window_control_area(WindowControlArea::Drag),
        )
        .child(chrome_control(
            "console-min",
            "minus",
            false,
            |window, _| window.minimize_window(),
        ))
        .child(chrome_control("console-close", "x", true, |window, _| {
            window.remove_window()
        }))
        .into_any_element()
}
