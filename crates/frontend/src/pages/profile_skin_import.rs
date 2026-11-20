//! Copying another player's skin by nickname: a field above the preset grid.
//!
//! The master does the lookup (its own players first, Mojang second) and sends
//! the model along with the picture, so the launcher only passes the name on.
//! The field is drawn by hand like the rename field next to it: GPUI has no
//! text input.

use super::common::Cx;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent};
use i18n::t;

/// The longest Minecraft nickname.
const MAX_NAME: usize = 16;

pub(super) fn import_row(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let query = ui.skin_import_query.clone();
    let busy = ui.skin_uploading;
    let ready = !busy && !query.is_empty();

    let field = div()
        .id("skin-import-field")
        .when_some(ui.skin_import_focus.clone(), |d, focus| {
            d.track_focus(&focus)
        })
        .flex_1()
        .h(px(32.))
        .px(px(12.))
        .rounded(px(R_SM))
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(BORDER))
        .focus(|s| s.border_color(rgb(ACCENT)))
        .flex()
        .items_center()
        .gap(px(8.))
        .cursor_text()
        .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
            let focus = this
                .skin_import_focus
                .get_or_insert_with(|| cx.focus_handle())
                .clone();
            focus.focus(window, cx);
            cx.notify();
        }))
        .on_key_down(cx.listener(import_key))
        .child(ic("user", 14., TEXT_MUTED))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(if query.is_empty() {
                    TEXT_MUTED
                } else {
                    TEXT_PRIMARY
                }))
                .child(if query.is_empty() {
                    t("profile-skin-import-placeholder")
                } else {
                    format!("{query}_")
                }),
        );

    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(field)
        .child(
            div()
                .id("skin-import-go")
                .h(px(32.))
                .px(px(12.))
                .rounded(px(R_SM))
                .bg(rgb(BG_CARD))
                .border_1()
                .border_color(rgb(BORDER))
                .flex()
                .items_center()
                .gap(px(8.))
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(if ready { TEXT_PRIMARY } else { TEXT_MUTED }))
                .when(ready, |d| {
                    d.cursor_pointer()
                        .hover(|s| s.bg(rgb(BG_INPUT)).border_color(rgb(CTA)))
                        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                            this.import_skin_by_name();
                            cx.notify();
                        }))
                })
                .child(ic("download", 14., if ready { CTA } else { TEXT_MUTED }))
                .child(if busy {
                    t("profile-skin-loading")
                } else {
                    t("profile-skin-import")
                }),
        )
        .into_any_element()
}

/// Enter copies, Escape clears. Anything that can't be in a nickname is
/// dropped as it is typed, so the master never has to refuse the format.
fn import_key(
    this: &mut LauncherUI,
    event: &gpui::KeyDownEvent,
    _w: &mut gpui::Window,
    cx: &mut gpui::Context<LauncherUI>,
) {
    if let Some(text) = super::common::pasted(event, cx) {
        accept(&mut this.skin_import_query, &text);
        cx.notify();
        return;
    }
    match event.keystroke.key.as_str() {
        "enter" => this.import_skin_by_name(),
        "escape" => this.skin_import_query.clear(),
        "backspace" => {
            this.skin_import_query.pop();
        }
        // `key_char` already accounts for layout and shift, and is empty under
        // cmd/ctrl, so shortcuts don't end up typed into the name.
        _ => {
            if let Some(ch) = event.keystroke.key_char.as_deref() {
                accept(&mut this.skin_import_query, ch);
            }
        }
    }
    cx.notify();
}

/// Appends what fits a Minecraft nickname: latin letters, digits and `_`, up
/// to sixteen of them.
fn accept(draft: &mut String, typed: &str) {
    for ch in typed.chars() {
        if draft.len() >= MAX_NAME {
            break;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' {
            draft.push(ch);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::accept;

    #[test]
    fn only_nickname_characters_are_kept() {
        let mut draft = String::new();
        accept(&mut draft, "jeb_ ");
        accept(&mut draft, "Стив/../");
        assert_eq!(draft, "jeb_");
    }

    #[test]
    fn a_paste_is_cut_at_sixteen() {
        let mut draft = String::from("abc");
        accept(&mut draft, "defghijklmnopqrstuvwxyz");
        assert_eq!(draft, "abcdefghijklmnop");
    }
}
