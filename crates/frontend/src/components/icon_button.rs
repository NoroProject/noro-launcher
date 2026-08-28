use crate::icons::ic;
use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, App, ClickEvent, ElementId, FontWeight, IntoElement, SharedString,
    Window,
};

#[allow(dead_code)]
pub fn icon_btn(
    id: impl Into<ElementId>,
    icon: &'static str,
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
        .gap(px(8.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .bg(rgb(if primary { ACCENT } else { BG_CARD }))
        .border_1()
        .border_color(rgb(if primary { ACCENT } else { BORDER }))
        .hover(move |s| s.bg(rgb(if primary { ACCENT_HOV } else { BG_CARD_HOV })))
        .text_color(rgb(if primary { 0xffffff } else { TEXT_PRIMARY }))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(16.))
        .font_weight(FontWeight::MEDIUM)
        .child(ic(icon, 16., if primary { 0xffffff } else { TEXT_PRIMARY }))
        .child(label.into())
        .on_click(on_click)
}

/// Step button for numeric fields. Smaller than the usual square icon button on
/// purpose — at 52px the memory row grew to nearly a hundred pixels tall.
pub fn stepper_btn(
    id: impl Into<ElementId>,
    icon: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .size(px(32.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(R_SM))
        .cursor_pointer()
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(BORDER))
        .hover(|d| d.bg(rgb(BG_CARD_HOV)).border_color(rgb(CTA)))
        .child(ic(icon, 14., TEXT_SECONDARY))
        .on_click(on_click)
}
