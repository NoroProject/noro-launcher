//! Building blocks for the settings screen: the setting row and its controls.

use super::common::Cx;
use crate::components::stepper_btn;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};

pub fn row(
    icon: &'static str,
    title: impl Into<SharedString>,
    hint: impl Into<SharedString>,
    control: AnyElement,
    divider: bool,
) -> AnyElement {
    div()
        .px(px(20.))
        .py(px(16.))
        .flex()
        .items_center()
        .gap(px(16.))
        .when(divider, |d| d.border_b_1().border_color(rgb(BORDER)))
        .child(
            div()
                .size(px(32.))
                .flex_shrink_0()
                .rounded(px(R_SM))
                .bg(rgba(0xffffff08))
                .flex()
                .items_center()
                .justify_center()
                .child(ic(icon, 16., TEXT_MUTED)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(13.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(TEXT_PRIMARY))
                        .child(title.into()),
                )
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(11.))
                        .text_color(rgb(TEXT_MUTED))
                        .child(hint.into()),
                ),
        )
        .child(div().min_w_0().child(control))
        .into_any_element()
}

/// A number with arrows on either side: `[−] 2048 MB [+]`.
pub fn stepper(
    id: &'static str,
    label: impl Into<SharedString>,
    value: u32,
    on_step: impl Fn(&mut LauncherUI, i32) + Clone + 'static,
    cx: &mut Cx,
) -> AnyElement {
    let dec = on_step.clone();
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .text_color(rgb(TEXT_MUTED))
                .child(label.into()),
        )
        .child(stepper_btn(
            SharedString::from(format!("{id}-dec")),
            "minus",
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                dec(this, -512);
                cx.notify();
            }),
        ))
        .child(
            div()
                .w(px(80.))
                .text_center()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(13.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(TEXT_PRIMARY))
                .child(format!("{value} MB")),
        )
        .child(stepper_btn(
            SharedString::from(format!("{id}-inc")),
            "plus",
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                on_step(this, 512);
                cx.notify();
            }),
        ))
        .into_any_element()
}

/// A path or a set of flags. Empty falls back to the placeholder, dimmed.
pub fn mono_value(value: &str, placeholder: &'static str) -> AnyElement {
    let empty = value.trim().is_empty();
    div()
        .max_w(px(320.))
        .truncate()
        .font_family("Courier New")
        .text_size(px(12.))
        .text_color(rgb(if empty { TEXT_MUTED } else { TEXT_SECONDARY }))
        .child(if empty {
            placeholder.to_string()
        } else {
            value.to_string()
        })
        .into_any_element()
}
