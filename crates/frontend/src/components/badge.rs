use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, IntoElement, SharedString};

pub fn badge(text: impl Into<SharedString>, color: u32) -> impl IntoElement {
    div()
        .px(px(8.))
        .py(px(2.))
        .rounded(px(R_SM))
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(color))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(12.))
        .text_color(rgb(color))
        .child(text.into())
}
