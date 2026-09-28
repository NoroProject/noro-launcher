use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, App, ClickEvent, ElementId, Window};

pub fn mod_toggle(
    id: impl Into<ElementId>,
    enabled: bool,
    allowed: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.into())
        .w(px(40.))
        .h(px(22.))
        .rounded_full()
        .flex_shrink_0()
        // Кремовый, а не зелёный: включённый мод — это не «всё хорошо», а
        // выбор игрока, и зелёный светофор среди кремовых акцентов выглядел
        // деталью из другого интерфейса.
        .bg(if enabled { rgb(CTA) } else { rgb(BG_INPUT) })
        .border_1()
        .border_color(if enabled { rgb(CTA) } else { rgb(BORDER) })
        .relative()
        .when(allowed, |d| d.cursor_pointer().on_click(on_click))
        .when(!allowed, |d| d.opacity(0.35))
        .child(
            div()
                .absolute()
                .top(px(2.))
                .left(if enabled { px(20.) } else { px(2.) })
                .size(px(16.))
                .rounded_full()
                .bg(if enabled {
                    rgb(ON_CTA)
                } else {
                    rgb(TEXT_MUTED)
                }),
        )
}
