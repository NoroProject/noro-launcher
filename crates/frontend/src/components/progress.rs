//! Горизонтальный прогресс-бар 0..1 с маджентовой заливкой.

use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, IntoElement};

pub fn progress_bar(fraction: f32) -> impl IntoElement {
    let f = fraction.clamp(0.0, 1.0);
    div()
        .w_full()
        .h(px(12.))
        .rounded(px(4.))
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(BORDER))
        .overflow_hidden()
        .child(
            div()
                .h_full()
                .w(gpui::relative(f))
                .bg(rgb(ACCENT))
                .rounded(px(4.)),
        )
}
