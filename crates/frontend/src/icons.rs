//! Иконки из `assets/icons/<name>.svg` (встроены в бинарник).

use gpui::{prelude::*, px, rgb, svg};

pub fn ic(name: &'static str, size: f32, color: u32) -> impl IntoElement {
    svg()
        .path(format!("icons/{name}.svg"))
        .size(px(size))
        .text_color(rgb(color))
}
