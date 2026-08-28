use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, App, ClickEvent, ElementId, IntoElement, Window};

pub fn checkbox_row(
    id: impl Into<ElementId>,
    checked: bool,
    enabled: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .size(px(20.))
        .flex_shrink_0()
        .rounded(px(4.))
        .border_1()
        .border_color(rgb(if checked { CTA } else { BORDER }))
        .bg(rgb(if checked { CTA } else { BG_INPUT }))
        .flex()
        .items_center()
        .justify_center()
        .when(enabled, |d| d.cursor_pointer().on_click(on_click))
        .when(!enabled, |d| d.opacity(0.4))
        .when(checked, |d| {
            d.child(div().text_color(rgb(ON_CTA)).text_xs().child("✓"))
        })
}
