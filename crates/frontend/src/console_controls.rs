//! The pieces the console's toolbar is made of.

use crate::console_window::ConsoleWindow;
use crate::icons::ic;
use crate::theme::*;
use bridge::ConsoleSettings;
use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, Context, FontWeight, KeyDownEvent,
};
use i18n::t;

type Cx<'a> = Context<'a, ConsoleWindow>;

/// A square icon button; `active` lights it the way the launcher lights a
/// switched-on icon button.
pub fn icon_button(
    id: &'static str,
    icon: &'static str,
    hint: String,
    active: bool,
    on_click: impl Fn(&mut ConsoleWindow, &mut Cx) + 'static,
    cx: &mut Cx,
) -> AnyElement {
    div()
        .id(id)
        .tooltip(crate::components::hint(hint))
        .size(px(32.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(R_SM))
        .cursor_pointer()
        .bg(if active {
            rgba((CTA << 8) | 0x18)
        } else {
            rgb(BG_INPUT)
        })
        .border_1()
        .border_color(rgb(if active { CTA } else { BORDER }))
        .hover(|d| d.bg(rgb(BG_CARD_HOV)))
        .child(ic(icon, 14., if active { CTA } else { TEXT_SECONDARY }))
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            on_click(this, cx);
            cx.notify();
        }))
        .into_any_element()
}

/// A level filter, with how many lines of that level the log holds.
pub fn level_chip(
    id: &'static str,
    label: &'static str,
    count: usize,
    color: u32,
    active: bool,
    flip: fn(&mut ConsoleSettings),
    cx: &mut Cx,
) -> AnyElement {
    let tone = if active { color } else { TEXT_MUTED };
    div()
        .id(id)
        .h(px(32.))
        .px(px(12.))
        .flex_none()
        .flex()
        .items_center()
        .gap(px(8.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .bg(if active {
            rgba((color << 8) | 0x18)
        } else {
            rgb(BG_INPUT)
        })
        .border_1()
        .border_color(if active {
            rgba((color << 8) | 0x66)
        } else {
            rgb(BORDER)
        })
        .child(div().size(px(8.)).rounded_full().bg(rgb(tone)))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(tone))
                .child(label),
        )
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(10.))
                .text_color(rgb(TEXT_MUTED))
                .child(count.to_string()),
        )
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.change_settings(flip, cx);
            cx.notify();
        }))
        .into_any_element()
}

/// GPUI has no text input: the launcher's usual focus handle and key handler,
/// with paste — a search is usually copied out of the log itself.
pub fn search_field(view: &mut ConsoleWindow, cx: &mut Cx) -> AnyElement {
    let focus = view
        .search_focus
        .get_or_insert_with(|| cx.focus_handle())
        .clone();
    let focus_for_click = focus.clone();
    let text = view.search.clone();
    div()
        .id("console-search")
        .track_focus(&focus)
        .w(px(220.))
        .h(px(32.))
        .flex_none()
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(8.))
        .rounded(px(R_SM))
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(BORDER))
        .focus(|s| s.border_color(rgb(ACCENT)))
        .cursor_text()
        .on_click(cx.listener(move |_this, _e: &ClickEvent, window, cx| {
            focus_for_click.focus(window, cx);
            cx.notify();
        }))
        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _w, cx| {
            type_key(this, event, cx);
            cx.notify();
        }))
        .child(ic("search", 13., TEXT_MUTED))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(px(12.))
                .text_color(rgb(if text.is_empty() {
                    TEXT_MUTED
                } else {
                    TEXT_PRIMARY
                }))
                .child(if text.is_empty() {
                    t("console-search")
                } else {
                    format!("{text}_")
                }),
        )
        .into_any_element()
}

fn type_key(view: &mut ConsoleWindow, event: &KeyDownEvent, cx: &mut Cx) {
    let keystroke = &event.keystroke;
    let mut search = view.search.clone();
    if keystroke.modifiers.secondary() {
        if keystroke.key == "v" {
            if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                // A search is one line; a pasted block would never match.
                search.push_str(text.lines().next().unwrap_or_default());
            }
        }
    } else {
        match keystroke.key.as_str() {
            "backspace" => {
                search.pop();
            }
            "escape" => search.clear(),
            "space" => search.push(' '),
            _ => {
                if let Some(ch) = keystroke.key_char.as_deref() {
                    if !ch.chars().any(char::is_control) {
                        search.push_str(ch);
                    }
                }
            }
        }
    }
    view.set_search(search);
}
