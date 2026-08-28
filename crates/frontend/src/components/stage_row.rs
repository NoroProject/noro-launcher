//! One sync stage: label, bar, megabytes. Stages run in parallel, so several of
//! these sit under the overall progress bar at once — hence the muted colours.

use crate::components::slim_progress_bar;
use crate::theme::*;
use bridge::SyncStage;
use gpui::{div, prelude::*, px, rgb, AnyElement};

pub fn stage_row(stage: SyncStage, done: u64, total: u64) -> AnyElement {
    let fraction = if total == 0 {
        0.0
    } else {
        (done as f32 / total as f32).clamp(0.0, 1.0)
    };
    let complete = total > 0 && done >= total;

    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .w(px(72.))
                .flex_shrink_0()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .text_color(rgb(if complete { TEXT_MUTED } else { TEXT_PRIMARY }))
                .child(stage.short_label()),
        )
        .child(div().flex_1().min_w_0().child(slim_progress_bar(fraction)))
        .child(
            div()
                .w(px(64.))
                .flex_shrink_0()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .text_color(rgb(TEXT_MUTED))
                .child(size_label(done, total, complete)),
        )
        .into_any_element()
}

/// A finished stage reads "done" rather than "88 / 88 MB" — there's nothing
/// left to compare.
fn size_label(done: u64, total: u64, complete: bool) -> String {
    if complete {
        return "done".into();
    }
    format!(
        "{:.0} / {:.0} MB",
        done as f64 / 1_048_576.0,
        total as f64 / 1_048_576.0
    )
}
