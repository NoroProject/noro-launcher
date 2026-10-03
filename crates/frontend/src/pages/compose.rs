//! The one text box that both the ticket thread and a conversation use.
//!
//! GPUI has no text input of its own, so a field here is a focus handle, a
//! key-down handler and a caret drawn as an underscore. That is fiddly enough
//! to be worth having once: the two screens that type into the master would
//! otherwise carry two copies, and the second copy is where the shift key and
//! the paste shortcut get forgotten.

use super::common::Cx;
use crate::components::btn;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent};
use i18n::t;

/// `send` runs with the typed text once the player presses enter or the button.
/// It never runs with an empty string.
pub fn box_for(
    ui: &LauncherUI,
    id: &'static str,
    send: impl Fn(&mut LauncherUI, String) + Clone + 'static,
    cx: &mut Cx,
) -> AnyElement {
    let text = ui.compose.clone();
    let focus = ui
        .compose_focus
        .clone()
        .unwrap_or_else(|| cx.focus_handle());
    let focus_for_click = focus.clone();
    let send_on_key = send.clone();

    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .id(id)
                .track_focus(&focus)
                .flex_1()
                .h(px(40.))
                .px(px(14.))
                .rounded(px(R_SM))
                .bg(rgb(BG_INPUT))
                .border_1()
                .border_color(rgb(BORDER))
                .focus(|s| s.border_color(rgb(ACCENT)))
                .flex()
                .items_center()
                .cursor_text()
                .on_click(cx.listener(move |this, _e: &ClickEvent, window, cx| {
                    if this.compose_focus.is_none() {
                        this.compose_focus = Some(focus_for_click.clone());
                    }
                    focus_for_click.focus(window, cx);
                    cx.notify();
                }))
                .on_key_down(
                    cx.listener(move |this, event: &gpui::KeyDownEvent, _w, cx| {
                        if let Some(text) = super::common::pasted(event, cx) {
                            this.compose.push_str(&text);
                            cx.notify();
                            return;
                        }
                        match event.keystroke.key.as_str() {
                            "backspace" => {
                                this.compose.pop();
                            }
                            "enter" => {
                                let text = this.compose.trim().to_string();
                                if !text.is_empty() {
                                    this.compose.clear();
                                    send_on_key(this, text);
                                }
                            }
                            "space" => this.compose.push(' '),
                            // `key_char` already accounts for shift and layout,
                            // and is empty under cmd/ctrl, so shortcuts are not
                            // typed into the message.
                            _ => {
                                if let Some(ch) = event.keystroke.key_char.as_deref() {
                                    this.compose.push_str(ch);
                                }
                            }
                        }
                        cx.notify();
                    }),
                )
                .child(
                    div()
                        .flex_1()
                        .text_size(px(12.))
                        .text_color(rgb(if text.is_empty() {
                            TEXT_MUTED
                        } else {
                            TEXT_PRIMARY
                        }))
                        .child(if text.is_empty() {
                            t("account-compose-placeholder")
                        } else {
                            format!("{text}_")
                        }),
                ),
        )
        .child(btn(
            "compose-send",
            t("account-send"),
            true,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                let text = this.compose.trim().to_string();
                if !text.is_empty() {
                    this.compose.clear();
                    send(this, text);
                }
                cx.notify();
            }),
        ))
        .into_any_element()
}
