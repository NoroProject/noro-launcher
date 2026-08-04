use crate::state::ConsoleWindow;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent, Context, FontWeight};

pub fn toggle(
    label: &'static str,
    active: bool,
    on_toggle: impl Fn(&mut ConsoleWindow) + Send + Sync + 'static,
    cx: &mut Context<ConsoleWindow>,
) -> AnyElement {
    div()
        .id(label)
        .cursor_pointer()
        .px(px(10.))
        .h(px(32.))
        .flex()
        .items_center()
        .rounded(px(R_SM))
        .bg(rgb(if active { CTA } else { BG_INPUT }))
        .border_1()
        .border_color(rgb(if active { CTA } else { BORDER }))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(12.))
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(if active { BG_WINDOW } else { TEXT_SECONDARY }))
        .child(label)
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            on_toggle(this);
            cx.notify();
        }))
        .into_any_element()
}

pub fn action(
    label: &'static str,
    on_click: impl Fn(&mut ConsoleWindow, &mut Context<ConsoleWindow>) + Send + Sync + 'static,
    cx: &mut Context<ConsoleWindow>,
) -> AnyElement {
    div()
        .id(label)
        .cursor_pointer()
        .px(px(10.))
        .h(px(32.))
        .flex()
        .items_center()
        .rounded(px(R_SM))
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(BORDER))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(12.))
        .text_color(rgb(TEXT_SECONDARY))
        .hover(|d| d.bg(rgb(BG_CARD_HOV)).text_color(rgb(TEXT_PRIMARY)))
        .child(label)
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            on_click(this, cx);
            cx.notify();
        }))
        .into_any_element()
}

pub fn clipboard_text(cx: &mut Context<ConsoleWindow>) -> Option<String> {
    cx.read_from_clipboard()
        .and_then(|item| item.text().map(|s| s.to_string()))
}
