use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, App, ClickEvent, ElementId, FontWeight, IntoElement, SharedString,
    Window,
};

pub fn btn(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    primary: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .px(px(16.))
        .h(px(40.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(R_SM))
        .cursor_pointer()
        .bg(rgb(if primary { CTA } else { BG_CARD }))
        .border_1()
        .border_color(rgb(if primary { CTA_HOV } else { BORDER }))
        .hover(move |s| s.bg(rgb(if primary { CTA_HOV } else { BG_CARD_HOV })))
        .text_color(rgb(if primary { ON_CTA } else { TEXT_PRIMARY }))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(16.))
        .font_weight(FontWeight::MEDIUM)
        .child(label.into())
        .on_click(on_click)
}
