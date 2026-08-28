use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, FontWeight, IntoElement, SharedString};

pub fn pixel_title(text: impl Into<SharedString>, size: f32, color: u32) -> impl IntoElement {
    div()
        .font_family(FONT_PIXEL)
        .font_weight(FontWeight::NORMAL)
        .text_size(px(size))
        .line_height(px(size * 1.4))
        .text_color(rgb(color))
        .child(text.into())
}

#[allow(dead_code)]
pub fn pixel_label(text: impl Into<SharedString>, color: u32) -> impl IntoElement {
    div()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(12.))
        .text_color(rgb(color))
        .child(text.into())
}
