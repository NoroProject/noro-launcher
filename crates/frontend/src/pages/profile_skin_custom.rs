//! A preset the player saved: its card, renaming it and deleting it.

use super::common::Cx;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, img, prelude::*, px, rgb, AnyElement, SharedString};
use i18n::t;

pub(super) fn custom_preset_card(
    ui: &LauncherUI,
    preset: &crate::state::SavedSkinPreset,
    cx: &mut Cx,
) -> AnyElement {
    let bytes = preset.bytes.clone();
    let id = preset.id.clone();
    let name = preset.name.clone();
    let is_active = ui.skin_bytes.as_deref() == Some(bytes.as_slice());

    let edit_id: SharedString = format!("edit-{}", id).into();
    let del_id: SharedString = format!("del-{}", id).into();
    let apply_id: SharedString = format!("apply-{}", id).into();

    let border_clr = if is_active { CTA_HOV } else { BORDER };
    let edit_preset_id = id.clone();
    let apply_bytes = bytes.clone();
    let card_apply_bytes = bytes.clone();

    let img_el = if let Some(loaded_img) = ui.preset_images.get(&id) {
        img(loaded_img.clone())
            .w(px(72.))
            .h(px(96.))
            .object_fit(gpui::ObjectFit::Contain)
            .into_any_element()
    } else {
        div()
            .w(px(72.))
            .h(px(96.))
            .flex()
            .items_center()
            .justify_center()
            .child(ic("user", 24., if is_active { CTA } else { TEXT_MUTED }))
            .into_any_element()
    };

    let edit_preset_id_del = id.clone();
    // Deleting can't be undone and the cross sits right next to the rename
    // button: the first click asks, the second deletes.
    let delete_armed = ui.is_armed(&format!("preset-del-{id}"));

    div()
        .id(SharedString::from(id.clone()))
        .relative()
        .w(gpui::relative(0.315))
        .p(px(4.))
        .bg(rgb(BG_CARD))
        .rounded(px(R_SM))
        .border_1()
        .border_color(rgb(border_clr))
        .hover(|s| s.bg(rgb(BG_INPUT)))
        .cursor_pointer()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.))
        .on_click(cx.listener(move |this, _, _, cx| {
            if !card_apply_bytes.is_empty() {
                this.upload_skin(card_apply_bytes.to_vec());
                cx.notify();
            }
        }))
        .child(
            div()
                .absolute()
                .top(px(4.))
                .right(px(4.))
                .flex()
                .gap(px(2.))
                .child(
                    div()
                        .id(edit_id)
                        .px(px(4.))
                        .py(px(2.))
                        .rounded(px(R_SM))
                        .bg(rgb(BG_INPUT))
                        .hover(|s| s.bg(rgb(BG_CARD)))
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(9.))
                        .text_color(rgb(TEXT_MUTED))
                        .child("✏️")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.renaming_preset = Some((edit_preset_id.clone(), name.clone()));
                            let focus = this
                                .rename_focus
                                .get_or_insert_with(|| cx.focus_handle())
                                .clone();
                            focus.focus(window, cx);
                            cx.notify();
                        })),
                )
                .child(
                    div()
                        .id(del_id)
                        .px(px(4.))
                        .py(px(2.))
                        .rounded(px(R_SM))
                        .bg(rgb(if delete_armed { ERROR } else { BG_INPUT }))
                        .hover(|s| s.bg(rgb(BG_CARD)))
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(9.))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(rgb(if delete_armed {
                            TEXT_PRIMARY
                        } else {
                            TEXT_MUTED
                        }))
                        .tooltip(crate::components::hint(t(if delete_armed {
                            "common-click-again"
                        } else {
                            "common-delete"
                        })))
                        .child(if delete_armed { "?" } else { "✕" })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if this.confirm_or_arm(format!("preset-del-{edit_preset_id_del}"), cx) {
                                this.custom_presets.retain(|p| p.id != edit_preset_id_del);
                            }
                            cx.notify();
                        })),
                ),
        )
        .child(img_el)
        .child(name_row(ui, preset, is_active, cx))
        .child(if is_active {
            div()
                .w_full()
                .py(px(3.))
                .rounded(px(R_SM))
                .bg(rgb(CTA))
                .flex()
                .items_center()
                .justify_center()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(9.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(ON_CTA))
                .child(t("profile-preset-current"))
                .into_any_element()
        } else {
            div()
                .id(apply_id)
                .w_full()
                .py(px(3.))
                .rounded(px(R_SM))
                .bg(rgb(BG_INPUT))
                .border_1()
                .border_color(rgb(BORDER))
                .hover(|s| s.bg(rgb(BG_CARD)))
                .flex()
                .items_center()
                .justify_center()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(9.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(TEXT_PRIMARY))
                .child(t("profile-preset-wear"))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.upload_skin(apply_bytes.to_vec());
                    cx.notify();
                }))
                .into_any_element()
        })
        .into_any_element()
}

/// The preset name, or a text field in its place while renaming.
///
/// The field is inline because GPUI gives us no text-input dialog on any of the
/// three platforms.
fn name_row(
    ui: &LauncherUI,
    preset: &crate::state::SavedSkinPreset,
    is_active: bool,
    cx: &mut Cx,
) -> AnyElement {
    let editing = ui
        .renaming_preset
        .as_ref()
        .filter(|(id, _)| id == &preset.id)
        .map(|(_, draft)| draft.clone());

    let Some(draft) = editing else {
        return div()
            .w_full()
            .px(px(4.))
            .truncate()
            .text_center()
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(10.))
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(rgb(if is_active { CTA } else { TEXT_PRIMARY }))
            .child(preset.name.clone())
            .into_any_element();
    };

    let focus = ui.rename_focus.clone().unwrap_or_else(|| cx.focus_handle());
    div()
        .id(SharedString::from(format!("rename-{}", preset.id)))
        .track_focus(&focus)
        .w_full()
        .px(px(4.))
        .py(px(2.))
        .rounded(px(R_SM))
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(ACCENT))
        .text_center()
        .truncate()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(10.))
        .text_color(rgb(TEXT_PRIMARY))
        .cursor_text()
        // The whole card is clickable, so a click into the field would
        // otherwise put the skin on.
        .on_click(|_, _, cx| cx.stop_propagation())
        .on_key_down(cx.listener(rename_key))
        .child(if draft.is_empty() {
            t("profile-skin-untitled")
        } else {
            draft
        })
        .into_any_element()
}

/// Enter saves, Escape cancels. An empty name is discarded rather than stored.
fn rename_key(
    this: &mut LauncherUI,
    event: &gpui::KeyDownEvent,
    _w: &mut gpui::Window,
    cx: &mut gpui::Context<LauncherUI>,
) {
    let pasted = super::common::pasted(event, cx);
    let Some((id, draft)) = this.renaming_preset.as_mut() else {
        return;
    };
    if let Some(text) = pasted {
        draft.push_str(&text);
        cx.notify();
        return;
    }
    match event.keystroke.key.as_str() {
        "escape" => this.renaming_preset = None,
        "backspace" => {
            draft.pop();
        }
        "space" => draft.push(' '),
        "enter" => {
            let name = draft.trim().to_string();
            let id = id.clone();
            if !name.is_empty() {
                if let Some(p) = this.custom_presets.iter_mut().find(|p| p.id == id) {
                    p.name = name;
                }
            }
            this.renaming_preset = None;
        }
        // `key_char` already accounts for layout and shift, and is empty under
        // cmd/ctrl, so shortcuts don't end up typed into the name.
        _ => {
            if let Some(ch) = event.keystroke.key_char.as_deref() {
                draft.push_str(ch);
            }
        }
    }
    cx.notify();
}
