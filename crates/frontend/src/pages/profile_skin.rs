//! Skin preview card: the turning figure, drag-to-rotate and the upload button.

use super::common::Cx;
use super::skin_drag;
use crate::components::btn;
use crate::state::{LauncherUI, Toast};
use crate::theme::*;
use gpui::{div, img, prelude::*, px, rgb, AnyElement, CursorStyle, MouseButton};
use i18n::t;
use schema::NotifLevel;

const PREVIEW_W: f32 = crate::skin::PREVIEW_W as f32;
const PREVIEW_H: f32 = crate::skin::PREVIEW_H as f32;

pub fn skin_card(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .w(px(PREVIEW_W))
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(preview_box(ui, cx))
        .when(is_grabbable(ui), |d| d.child(drag_hint()))
        .child(upload_button(cx))
        .into_any_element()
}

/// Крутить можно только настоящую 3D-модель — плоская текстура-фолбэк
/// на поворот не отзывается.
fn is_grabbable(ui: &LauncherUI) -> bool {
    ui.skin_bytes.is_some() && ui.skin_preview.is_some()
}

fn preview_box(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div()
        .id("skin-preview-area")
        .w(px(PREVIEW_W))
        .h(px(PREVIEW_H))
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
        .text_size(px(12.))
        .text_color(rgb(TEXT_MUTED))
        .child(t("profile-drag-to-rotate"))
        .into_any_element()
}

fn preview_content(ui: &LauncherUI) -> AnyElement {
    if ui.skin_loading || ui.skin_uploading {
        return placeholder(t("profile-skin-loading"));
    }
    if let Some(p) = &ui.skin_preview {
        return img(p.clone())
            .w(px(PREVIEW_W))
            .h(px(PREVIEW_H))
            .into_any_element();
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
    let text = text.into();
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(14.))
        .text_color(rgb(TEXT_MUTED))
        .child(text)
        .into_any_element()
}

fn upload_button(cx: &mut Cx) -> AnyElement {
    btn(
        "upload-skin",
        t("profile-upload-skin"),
        true,
        cx.listener(on_upload_click),
    )
    .into_any_element()
}

fn on_upload_click(
    this: &mut LauncherUI,
    _e: &gpui::ClickEvent,
    _w: &mut gpui::Window,
    cx: &mut gpui::Context<LauncherUI>,
) {
    let script = "POSIX path of (choose file of type {\"public.png\"} with prompt \"Select Minecraft skin PNG\")";
    if let Ok(output) = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
    {
        if output.status.success() {
            let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !p.is_empty() {
                if let Ok(b) = std::fs::read(&p) {
                    if b.len() > 8 && &b[0..8] == b"\x89PNG\r\n\x1a\n" && b.len() < 256 * 1024 {
                        this.upload_skin(b);
                        cx.notify();
                        return;
                    }
                }
            }
        }
    }
    this.toast = Some(Toast {
        text: t("profile-skin-invalid"),
        level: NotifLevel::Warning,
    });
    cx.notify();
}
