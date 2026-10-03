//! Skin preview card: turning figure, drag-to-rotate, presets and upload button.

use super::common::{panel, Cx};
use super::skin_drag;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, img, prelude::*, px, rgb, AnyElement, CursorStyle, MouseButton};
use i18n::t;

pub(super) const PREVIEW_W: f32 = crate::skin::PREVIEW_W as f32;
pub(super) const PREVIEW_H: f32 = crate::skin::PREVIEW_H as f32;

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

pub(super) fn is_grabbable(ui: &LauncherUI) -> bool {
    ui.skin_bytes.is_some() && ui.skin_preview.is_some()
}

pub(super) fn preview_box(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
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

pub(super) fn drag_hint() -> AnyElement {
    div()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(11.))
        .text_color(rgb(TEXT_MUTED))
        .child(t("profile-drag-to-rotate"))
        .into_any_element()
}

pub(super) fn preview_content(ui: &LauncherUI) -> AnyElement {
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

pub(super) fn placeholder(text: impl Into<gpui::SharedString>) -> AnyElement {
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
