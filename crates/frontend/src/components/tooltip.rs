//! Hover hints for buttons that are only an icon.
//!
//! There wasn't a single one: a row of glyphs in the sidebar and the game bar
//! had to be learnt by clicking each of them.

use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyView, App, Context, SharedString, Window};

struct Tooltip {
    text: SharedString,
}

impl Render for Tooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(8.))
            .py(px(4.))
            .rounded(px(R_SM))
            .bg(rgb(BG_PANEL))
            .border_1()
            .border_color(rgb(BORDER))
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(12.))
            .text_color(rgb(TEXT_PRIMARY))
            .child(self.text.clone())
    }
}

/// For `.tooltip(...)` on an element with an id.
pub fn hint(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let text = text.into();
    move |_window, cx| {
        let text = text.clone();
        cx.new(|_| Tooltip { text }).into()
    }
}
