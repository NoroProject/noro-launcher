//! One switch out of several — the second-level tabs, filters and pickers.
//!
//! There is one of these because the launcher had three. On the content screen
//! alone, «Каталог / Установленное» were dark with cream text, the content
//! types were magenta, and the providers were a filled block — three different
//! answers to «which one is on», in three rows stacked on top of each other.
//!
//! The rule this settles on:
//!
//! * **cream means an action or a choice in force**, and nothing else is cream;
//! * **green means a state that was reached** — installed, done — and is never
//!   something to press;
//! * an icon belongs to a row or to none of it: `Каталог / Установленное` are
//!   places and get one, `Modrinth / CurseForge` are names of services and do
//!   not, because a logo we do not have would be the only honest icon there.
//!
//! The page-level tabs stay louder (a filled cream block): they switch the
//! screen, these switch what is on it.

use crate::icons::ic;
use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, App, ClickEvent, ElementId, SharedString, Window,
};

/// `icon` of `None` leaves the label alone — pass it consistently within a row.
pub fn segment(
    id: impl Into<ElementId>,
    icon: Option<&'static str>,
    label: impl Into<SharedString>,
    active: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let colour = if active { CTA } else { TEXT_MUTED };

    div()
        .id(id.into())
        .h(px(32.))
        .px(px(12.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .gap(px(6.))
        .flex_shrink_0()
        .cursor_pointer()
        .bg(if active {
            rgba((CTA << 8) | 0x18)
        } else {
            rgba(0x00000000)
        })
        .border_1()
        .border_color(if active { rgb(CTA) } else { rgb(BORDER) })
        .hover(|d| d.bg(rgba(0xffffff0e)))
        .children(icon.map(|name| ic(name, 13., colour)))
        .child(
            div()
                .text_size(px(11.))
                .text_color(rgb(colour))
                .child(label.into()),
        )
        .on_click(on_click)
        .into_any_element()
}
