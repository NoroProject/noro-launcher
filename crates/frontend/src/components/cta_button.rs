use crate::icons::ic;
use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, App, ClickEvent, ElementId, FontWeight, IntoElement, SharedString,
    Window,
};

pub fn cta_button(
    id: impl Into<ElementId>,
    icon: Option<&'static str>,
    label: impl Into<SharedString>,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .h(px(56.))
        .w_full()
        .flex()
        .items_center()
        .justify_center()
        .gap(px(12.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .bg(rgb(CTA))
        .border_2()
        .border_color(rgb(0xfffbe8))
        .shadow_lg()
        .hover(|s| s.bg(rgb(CTA_HOV)))
        .text_color(rgb(ON_CTA))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(18.))
        .font_weight(FontWeight::BOLD)
        .when_some(icon, |d, name| d.child(ic(name, 20., ON_CTA)))
        .child(label.into())
        .on_click(on_click)
}
