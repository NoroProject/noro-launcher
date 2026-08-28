//! Icons come from `assets/icons/<name>.svg`, embedded in the binary.

use gpui::{prelude::*, px, rgb, svg};

pub fn ic(name: &'static str, size: f32, color: u32) -> impl IntoElement {
    svg()
        .path(format!("icons/{name}.svg"))
        .size(px(size))
        .text_color(rgb(color))
}
