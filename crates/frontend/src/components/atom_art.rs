//! Placeholder logo area for future launcher artwork.

use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement};

pub fn atom_art() -> AnyElement {
    div()
        .size_full()
        .relative()
        .overflow_hidden()
        .child(large_logo())
        .into_any_element()
}

pub fn tiny_atom_logo() -> AnyElement {
    div()
        .w(px(64.))
        .h(px(28.))
        .border_2()
        .border_color(rgb(CTA))
        .flex()
        .items_center()
        .justify_center()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(12.))
        .text_color(rgb(CTA))
        .child("LOGO")
        .into_any_element()
}

fn large_logo() -> AnyElement {
    div()
        .absolute()
        .right(px(112.))
        .top(px(128.))
        .w(px(320.))
        .h(px(320.))
        .border_4()
        .border_color(rgb(0x7388a8))
        .bg(rgba(0x13233d88))
        .flex()
        .items_center()
        .justify_center()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(28.))
        .text_color(rgb(CTA))
        .child("LOGO")
        .into_any_element()
}
