// Over 150 lines: the settings-row control, the dialog and its text field. The
// field only exists inside the dialog and the dialog only opens from the row,
// so apart they would be three files that can't be read without each other.
//! JVM flags of one server: the player's, which they edit, and the build's,
//! which they can only read.
//!
//! The build's flags are not a suggestion. GTNH, for one, does not start
//! without its own system class loader, so the backend adds them on every
//! launch, after the player's. The player can add to them, not take them away.

use super::common::Cx;
use crate::components::btn;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, ClipboardItem, FontWeight,
    KeyDownEvent, SharedString,
};
use i18n::t;
use uuid::Uuid;

/// The control in the settings row: what the player has set, and the way in.
pub fn control(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let own = ui.server_client_settings(server_id).jvm_flags;
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(
            div()
                .max_w(px(240.))
                .min_w_0()
                .font_family("Courier New")
                .text_size(px(12.))
                .text_color(rgb(if own.is_empty() {
                    TEXT_MUTED
                } else {
                    TEXT_SECONDARY
                }))
                .truncate()
                .child(if own.is_empty() {
                    t("settings-jvm-none")
                } else {
                    own.clone()
                }),
        )
        .child(
            div()
                .id("jvm-flags-edit")
                .h(px(32.))
                .px(px(12.))
                .rounded(px(R_SM))
                .bg(rgb(BG_CARD))
                .border_1()
                .border_color(rgb(BORDER))
                .hover(|d| d.bg(rgb(BG_CARD_HOV)).border_color(rgb(CTA)))
                .cursor_pointer()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(ic("code", 13., TEXT_SECONDARY))
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(10.))
                        .text_color(rgb(TEXT_SECONDARY))
                        .child(t("settings-jvm-edit")),
                )
                .on_click(cx.listener(move |this, _e: &ClickEvent, window, cx| {
                    this.jvm_flags_draft = one_per_line(&own);
                    this.jvm_flags_open = true;
                    // Typing works straight away, without a click into the field.
                    this.jvm_flags_focus
                        .get_or_insert_with(|| cx.focus_handle())
                        .focus(window, cx);
                    cx.notify();
                })),
        )
        .into_any_element()
}

/// Drawn at page level, like the Java list, so no later row paints over it.
pub fn dialog(ui: &mut LauncherUI, server_id: Uuid, cx: &mut Cx) -> Option<AnyElement> {
    if !ui.jvm_flags_open {
        return None;
    }
    let build = ui
        .server_recommendations
        .get(&server_id)
        .map(|s| s.jvm_flags.clone())
        .unwrap_or_default();
    let launched = format!("{} {build}", ui.jvm_flags_draft)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    Some(
        div()
            .id("jvm-flags-backdrop")
            .absolute()
            .inset_0()
            .bg(rgba(0x000000aa))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.jvm_flags_open = false;
                cx.notify();
            }))
            .child(
                div()
                    .id("jvm-flags-dialog")
                    .occlude()
                    .on_click(|_, _, _| {})
                    .w(px(640.))
                    // The window is never shorter than 680: the dialog fits, and
                    // whatever does not fit is the build's list, which scrolls.
                    .max_h(px(560.))
                    .flex()
                    .flex_col()
                    .rounded(px(R_LG))
                    .bg(rgb(BG_PANEL))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .child(header(launched, cx))
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .p(px(16.))
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .child(caption(t("settings-jvm-user")))
                            .child(field(ui, cx))
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(rgb(TEXT_MUTED))
                                    .child(t("settings-jvm-user-hint")),
                            )
                            .child(div().h(px(8.)))
                            .child(caption(t("settings-jvm-master")))
                            .child(build_flags(&build)),
                    )
                    .child(footer(server_id, cx)),
            )
            .into_any_element(),
    )
}

fn header(launched: String, cx: &mut Cx) -> AnyElement {
    div()
        .h(px(52.))
        .flex_none()
        .px(px(16.))
        .flex()
        .items_center()
        .gap(px(8.))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(ic("code", 15., ACCENT))
        .child(
            div()
                .flex_1()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(13.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(t("settings-jvm-flags")),
        )
        .child(
            div()
                .id("jvm-flags-copy")
                .h(px(28.))
                .px(px(12.))
                .rounded(px(R_SM))
                .bg(rgb(BG_CARD))
                .border_1()
                .border_color(rgb(BORDER))
                .hover(|d| d.bg(rgb(BG_CARD_HOV)))
                .flex()
                .items_center()
                .cursor_pointer()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(10.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(t("settings-jvm-copy"))
                // Everything in launch order: what the JVM actually gets.
                .on_click(move |_e, _w, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(launched.clone()));
                }),
        )
        .child(
            div()
                .id("jvm-flags-close")
                .size(px(28.))
                .rounded(px(R_SM))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|d| d.bg(rgba(0xffffff10)))
                .child(ic("x", 14., TEXT_MUTED))
                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.jvm_flags_open = false;
                    cx.notify();
                })),
        )
        .into_any_element()
}

fn caption(text: impl Into<SharedString>) -> AnyElement {
    div()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(11.))
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(TEXT_MUTED))
        .child(text.into())
        .into_any_element()
}

/// GPUI has no text input, so this is the same focus-handle-and-key-handler
/// field as `compose`, with line breaks and paste: flags are long and usually
/// copied from somewhere.
fn field(ui: &mut LauncherUI, cx: &mut Cx) -> AnyElement {
    let draft = ui.jvm_flags_draft.clone();
    let focus = ui
        .jvm_flags_focus
        .get_or_insert_with(|| cx.focus_handle())
        .clone();
    let focus_for_click = focus.clone();

    div()
        .id("jvm-flags-field")
        .track_focus(&focus)
        .flex_none()
        .min_h(px(96.))
        .max_h(px(160.))
        .overflow_y_scroll()
        .p(px(12.))
        .rounded(px(R_SM))
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(BORDER))
        .focus(|s| s.border_color(rgb(ACCENT)))
        .cursor_text()
        .font_family("Courier New")
        .text_size(px(12.))
        .text_color(rgb(if draft.is_empty() {
            TEXT_MUTED
        } else {
            TEXT_PRIMARY
        }))
        .on_click(cx.listener(move |_this, _e: &ClickEvent, window, cx| {
            focus_for_click.focus(window, cx);
            cx.notify();
        }))
        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _w, cx| {
            type_key(this, event, cx);
            cx.notify();
        }))
        .child(if draft.is_empty() {
            t("settings-jvm-user-placeholder")
        } else {
            format!("{draft}_")
        })
        .into_any_element()
}

fn type_key(ui: &mut LauncherUI, event: &KeyDownEvent, cx: &mut Cx) {
    let keystroke = &event.keystroke;
    if keystroke.modifiers.secondary() {
        if keystroke.key == "v" {
            if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                ui.jvm_flags_draft.push_str(&text);
            }
        }
        return;
    }
    match keystroke.key.as_str() {
        "backspace" => {
            ui.jvm_flags_draft.pop();
        }
        "enter" => ui.jvm_flags_draft.push('\n'),
        "space" => ui.jvm_flags_draft.push(' '),
        "escape" => ui.jvm_flags_open = false,
        // `key_char` already accounts for shift and the layout. Tab and the
        // like come through as control characters and have no place in a flag.
        _ => {
            if let Some(ch) = keystroke.key_char.as_deref() {
                if !ch.chars().any(char::is_control) {
                    ui.jvm_flags_draft.push_str(ch);
                }
            }
        }
    }
}

/// The only part of the dialog that scrolls: a build can bring dozens of
/// `--add-opens`, and the dialog has to stay inside the window.
fn build_flags(build: &str) -> AnyElement {
    let text = one_per_line(build);
    div()
        .id("jvm-flags-build")
        .flex_1()
        .min_h(px(48.))
        .overflow_y_scroll()
        .p(px(12.))
        .rounded(px(R_SM))
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(BORDER))
        .font_family("Courier New")
        .text_size(px(12.))
        .text_color(rgb(if text.is_empty() {
            TEXT_MUTED
        } else {
            TEXT_SECONDARY
        }))
        .child(if text.is_empty() {
            t("settings-jvm-empty")
        } else {
            text
        })
        .into_any_element()
}

fn footer(server_id: Uuid, cx: &mut Cx) -> AnyElement {
    div()
        .flex_none()
        .px(px(16.))
        .py(px(12.))
        .flex()
        .justify_end()
        .gap(px(8.))
        .border_t_1()
        .border_color(rgb(BORDER))
        .child(btn(
            "jvm-flags-cancel",
            t("common-cancel"),
            false,
            cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.jvm_flags_open = false;
                cx.notify();
            }),
        ))
        .child(btn(
            "jvm-flags-save",
            t("common-save"),
            true,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                // Saved as one line; the line breaks are only for reading.
                let flags = this
                    .jvm_flags_draft
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                this.set_server_jvm_flags(server_id, flags);
                this.jvm_flags_open = false;
                cx.notify();
            }),
        ))
        .into_any_element()
}

/// A flag per line, an option that takes a separate value kept on one line
/// with it: `--add-opens` alone on a line says nothing.
fn one_per_line(flags: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut tokens = flags.split_whitespace();
    while let Some(token) = tokens.next() {
        let takes_value = matches!(
            token,
            "--add-opens" | "--add-exports" | "--add-reads" | "--add-modules" | "-cp"
        );
        match tokens.clone().next() {
            Some(value) if takes_value => {
                lines.push(format!("{token} {value}"));
                tokens.next();
            }
            _ => lines.push(token.to_string()),
        }
    }
    lines.join("\n")
}
