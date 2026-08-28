use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, IntoElement};

pub fn progress_bar(fraction: f32) -> impl IntoElement {
    bar(fraction, 12.)
}

/// For the per-stage rows: several bars share that overlay and the full height
/// fills it.
pub fn slim_progress_bar(fraction: f32) -> impl IntoElement {
    bar(fraction, 8.)
}

fn bar(fraction: f32, height: f32) -> impl IntoElement {
    let f = fraction.clamp(0.0, 1.0);
    div()
        .w_full()
        .h(px(height))
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
