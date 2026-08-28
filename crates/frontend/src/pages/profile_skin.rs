//! Skin preview card: turning figure, drag-to-rotate, presets and upload button.

use super::common::{panel, Cx};
use super::profile_skin_pick::on_upload_click;
use super::skin_drag;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{
    div, img, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, CursorStyle, MouseButton,
    SharedString,
};
use i18n::t;

const PREVIEW_W: f32 = crate::skin::PREVIEW_W as f32;
const PREVIEW_H: f32 = crate::skin::PREVIEW_H as f32;

pub fn skin_card(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    panel()
        .p(px(12.))
        .w(gpui::relative(0.4))
        .h_full()
        .flex()
        .flex_col()
        .gap(px(10.))
        .child(preview_box(ui, cx))
        .when(is_grabbable(ui), |d| d.child(drag_hint()))
        .children(super::profile_skin_model::model_row(ui, cx))
        .into_any_element()
}

pub fn skin_presets_panel(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let presets = [
        ("Steve", "steve"),
        ("Alex", "alex"),
        ("Ari", "ari"),
        ("Zuri", "zuri"),
        ("Efe", "efe"),
        ("Makena", "makena"),
        ("Kai", "kai"),
        ("Sunny", "sunny"),
        ("Noor", "noor"),
    ];

    panel()
        .p(px(14.))
        .flex_1()
        .min_h_0()
        .h_full()
        .overflow_hidden()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(header_row())
        .child(
            div()
                .id("skin-presets-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .pb(px(12.))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(8.))
                        .px(px(2.))
                        .child(add_preset_tile_card(cx))
                        .children(
                            ui.custom_presets
                                .iter()
                                .map(|p| custom_preset_card(ui, p, cx)),
                        )
                        .children(
                            presets
                                .into_iter()
                                .map(|(name, id)| standard_preset_card(ui, name, id, cx)),
                        ),
                ),
        )
        .into_any_element()
}

fn add_preset_tile_card(cx: &mut Cx) -> AnyElement {
    div()
        .id("add-preset-tile")
        .w(gpui::relative(0.315))
        .p(px(6.))
        .bg(rgb(BG_CARD))
        .rounded(px(R_SM))
        .border_1()
        .border_color(rgb(BORDER))
        .hover(|s| s.bg(rgb(BG_INPUT)).border_color(rgb(CTA)))
        .cursor_pointer()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(8.))
        .py(px(24.))
        .on_click(cx.listener(on_upload_click))
        .child(
            div()
                .size(px(44.))
                .rounded_full()
                .bg(rgba(0xf3e7b31a))
                .border_1()
                .border_color(rgba(0xf3e7b344))
                .flex()
                .items_center()
                .justify_center()
                .child(ic("plus", 22., CTA)),
        )
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(TEXT_PRIMARY))
                .child(t("profile-preset-upload")),
        )
        .into_any_element()
}

fn header_row() -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(13.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(t("profile-presets-title")),
        )
        .into_any_element()
}

fn is_preset_active(ui: &LauncherUI, id: &str) -> bool {
    let lower_id = id.to_lowercase();
    if let Some(url) = &ui.skin_url {
        let lower_url = url.to_lowercase();
        if lower_url.contains(&format!("/presets/{}.png", lower_id))
            || lower_url.contains(&format!("preset={}", lower_id))
        {
            return true;
        }
    }
    if let Some(user) = &ui.user {
        if let Some(url) = &user.skin_url {
            let lower_url = url.to_lowercase();
            if lower_url.contains(&format!("/presets/{}.png", lower_id))
                || lower_url.contains(&format!("preset={}", lower_id))
            {
                return true;
            }
        }
    }
    false
}

fn custom_preset_card(
    ui: &LauncherUI,
    preset: &crate::state::SavedSkinPreset,
    cx: &mut Cx,
) -> AnyElement {
    let bytes = preset.bytes.clone();
    let id = preset.id.clone();
    let name = preset.name.clone();
    let is_active = ui.skin_bytes.as_ref() == Some(&bytes);

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
                this.upload_skin(card_apply_bytes.clone());
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
                        .bg(rgb(BG_INPUT))
                        .hover(|s| s.bg(rgb(BG_CARD)))
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(9.))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(rgb(TEXT_MUTED))
                        .child("✕")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.custom_presets.retain(|p| p.id != edit_preset_id_del);
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
                    this.upload_skin(apply_bytes.clone());
                    cx.notify();
                }))
                .into_any_element()
        })
        .into_any_element()
}

fn standard_preset_card(
    ui: &LauncherUI,
    name: &'static str,
    id: &'static str,
    cx: &mut Cx,
) -> AnyElement {
    let is_active = is_preset_active(ui, id);
    let border_clr = if is_active { CTA_HOV } else { BORDER };

    let img_el = if let Some(loaded_img) = ui.preset_images.get(id) {
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

    div()
        .id(id)
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
        .child(img_el)
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(10.))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(if is_active { CTA } else { TEXT_PRIMARY }))
                .child(name),
        )
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
                .into_any_element()
        })
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| apply_preset(this, id, cx)))
        .into_any_element()
}

fn apply_preset(this: &mut LauncherUI, name: &'static str, cx: &mut Cx) {
    let master_url = this.config.master_url.clone();
    let url = format!(
        "{}/api/textures/presets/{}.png",
        master_url.trim_end_matches('/'),
        name
    );

    cx.spawn(async move |this, cx| {
        let loaded = crate::image_loader::load_image_and_bytes(url).await;
        let _ = this.update(cx, |this, cx| {
            if let Ok((_, bytes)) = loaded {
                this.upload_skin(bytes);
            }
            cx.notify();
        });
    })
    .detach();
}

fn is_grabbable(ui: &LauncherUI) -> bool {
    ui.skin_bytes.is_some() && ui.skin_preview.is_some()
}

fn preview_box(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .id("skin-preview-area")
        .w_full()
        .flex_1()
        .min_h(px(PREVIEW_H))
        .bg(rgb(BG_INPUT))
        .rounded(px(R_SM))
        .border_1()
        .border_color(rgb(BORDER))
        .overflow_hidden()
        .flex()
        .items_center()
        .justify_center()
        .when(is_grabbable(ui), |d| {
            d.cursor(CursorStyle::OpenHand)
                .on_mouse_down(MouseButton::Left, cx.listener(skin_drag::on_grab))
        })
        .child(preview_content(ui))
        .into_any_element()
}

fn drag_hint() -> AnyElement {
    div()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(11.))
        .text_color(rgb(TEXT_MUTED))
        .child(t("profile-drag-to-rotate"))
        .into_any_element()
}

fn preview_content(ui: &LauncherUI) -> AnyElement {
    if let Some(p) = &ui.skin_preview {
        return img(p.clone())
            .w(px(PREVIEW_W))
            .h(px(PREVIEW_H))
            .into_any_element();
    }
    if ui.skin_loading || ui.skin_uploading {
        return placeholder(t("profile-skin-loading"));
    }
    if let Some(s) = &ui.skin_image {
        return img(s.clone())
            .w(px(PREVIEW_W))
            .h(px(PREVIEW_H))
            .into_any_element();
    }
    placeholder(t("profile-no-skin"))
}

fn placeholder(text: impl Into<gpui::SharedString>) -> AnyElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(13.))
        .text_color(rgb(TEXT_MUTED))
        .child(text.into())
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
    let Some((id, draft)) = this.renaming_preset.as_mut() else {
        return;
    };
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
