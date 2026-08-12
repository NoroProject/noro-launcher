//! Skin preview card: turning figure, drag-to-rotate, presets and upload button.

use super::common::{panel, Cx};
use super::skin_drag;
use crate::components::btn;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, img, prelude::*, px, rgb, AnyElement, ClickEvent, CursorStyle, MouseButton, SharedString};
use i18n::t;

const PREVIEW_W: f32 = crate::skin::PREVIEW_W as f32;
const PREVIEW_H: f32 = crate::skin::PREVIEW_H as f32;

pub fn skin_card(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    panel().p(px(10.)).w(px(168.)).h_full().flex_1().flex().flex_col().gap(px(8.))
        .child(preview_box(ui, cx))
        .when(is_grabbable(ui), |d| d.child(drag_hint()))
        .into_any_element()
}

pub fn skin_presets_panel(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let presets = [
        ("Steve", "steve"), ("Alex", "alex"), ("Ari", "ari"), ("Zuri", "zuri"),
        ("Efe", "efe"), ("Makena", "makena"), ("Kai", "kai"), ("Sunny", "sunny"),
        ("Noor", "noor"),
    ];

    panel().p(px(14.)).flex_1().min_h_0().h_full().overflow_hidden().flex().flex_col().gap(px(12.))
        .child(header_row())
        .child(div().id("skin-presets-scroll").flex_1().min_h_0().overflow_y_scroll().pb(px(12.))
            .child(div().flex().flex_wrap().gap(px(8.)).px(px(2.))
                .child(add_preset_tile_card(cx))
                .child(import_username_tile_card(cx))
                .children(ui.custom_presets.iter().map(|p| custom_preset_card(ui, p, cx)))
                .children(presets.into_iter().map(|(name, id)| standard_preset_card(ui, name, id, cx)))
            ))
        .into_any_element()
}

fn add_preset_tile_card(cx: &mut Cx) -> AnyElement {
    div().id("add-preset-tile").w(px(112.)).h(px(140.)).p(px(6.))
        .bg(rgb(BG_CARD)).rounded(px(R_SM)).border_1().border_color(rgb(CTA))
        .hover(|s| s.bg(rgb(BG_INPUT))).cursor_pointer()
        .flex().flex_col().items_center().justify_center().gap(px(4.))
        .child(div().w(px(36.)).h(px(36.)).rounded_full().bg(rgb(BG_INPUT)).flex().items_center().justify_center().child(ic("plus", 20., CTA)))
        .child(div().font_family(FONT_PIXEL_ALT).text_size(px(11.)).font_weight(gpui::FontWeight::BOLD).text_color(rgb(CTA)).child("Новый скин"))
        .child(div().font_family(FONT_PIXEL_ALT).text_size(px(9.)).text_color(rgb(TEXT_MUTED)).child("Загрузить .PNG"))
        .on_click(cx.listener(on_upload_click))
        .into_any_element()
}

fn import_username_tile_card(cx: &mut Cx) -> AnyElement {
    div().id("import-username-tile").w(px(112.)).h(px(140.)).p(px(6.))
        .bg(rgb(BG_CARD)).rounded(px(R_SM)).border_1().border_color(rgb(CTA))
        .hover(|s| s.bg(rgb(BG_INPUT))).cursor_pointer()
        .flex().flex_col().items_center().justify_center().gap(px(4.))
        .child(div().w(px(36.)).h(px(36.)).rounded_full().bg(rgb(BG_INPUT)).flex().items_center().justify_center().child(ic("user", 20., CTA)))
        .child(div().font_family(FONT_PIXEL_ALT).text_size(px(11.)).font_weight(gpui::FontWeight::BOLD).text_color(rgb(CTA)).child("По нику"))
        .child(div().font_family(FONT_PIXEL_ALT).text_size(px(9.)).text_color(rgb(TEXT_MUTED)).child("Импорт скина"))
        .on_click(cx.listener(on_import_by_username_click))
        .into_any_element()
}

fn header_row() -> AnyElement {
    div().flex().items_center().justify_between()
        .child(div().font_family(FONT_PIXEL_ALT).text_size(px(13.)).font_weight(gpui::FontWeight::BOLD).text_color(rgb(CTA)).child("Пресеты скинов"))
        .into_any_element()
}

fn is_preset_active(ui: &LauncherUI, id: &str) -> bool {
    let lower_id = id.to_lowercase();
    if let Some(url) = &ui.skin_url {
        let lower_url = url.to_lowercase();
        if lower_url.contains(&format!("/presets/{}.png", lower_id)) || lower_url.contains(&format!("preset={}", lower_id)) {
            return true;
        }
    }
    if let Some(user) = &ui.user {
        if let Some(url) = &user.skin_url {
            let lower_url = url.to_lowercase();
            if lower_url.contains(&format!("/presets/{}.png", lower_id)) || lower_url.contains(&format!("preset={}", lower_id)) {
                return true;
            }
        }
    }
    false
}

fn custom_preset_card(ui: &LauncherUI, preset: &crate::state::SavedSkinPreset, cx: &mut Cx) -> AnyElement {
    let bytes = preset.bytes.clone();
    let id = preset.id.clone();
    let name = preset.name.clone();
    let is_active = ui.skin_bytes.as_ref() == Some(&bytes);

    let apply_id: SharedString = format!("apply-{}", id).into();
    let edit_id: SharedString = format!("edit-{}", id).into();
    let del_id: SharedString = format!("del-{}", id).into();

    let border_clr = if is_active { CTA_HOV } else { BORDER };
    let edit_preset_id = id.clone();

    div().id(SharedString::from(id.clone())).w(px(112.)).p(px(6.)).bg(rgb(BG_CARD)).rounded(px(R_SM)).border_1().border_color(rgb(border_clr)).hover(|s| s.bg(rgb(BG_INPUT))).cursor_pointer().flex().flex_col().items_center().gap(px(4.))
        .child(div().w(px(72.)).h(px(85.)).flex().items_center().justify_center().child(ic("user", 24., if is_active { CTA } else { TEXT_MUTED })))
        .child(div().w_full().truncate().text_center().font_family(FONT_PIXEL_ALT).text_size(px(11.)).font_weight(gpui::FontWeight::BOLD).text_color(rgb(if is_active { CTA } else { TEXT_PRIMARY })).child(name.clone()))
        .child(div().flex().gap(px(4.))
            .child(if is_active {
                div().px(px(6.)).py(px(2.)).rounded(px(R_SM)).bg(rgb(CTA)).font_family(FONT_PIXEL_ALT).text_size(px(9.)).font_weight(gpui::FontWeight::BOLD).text_color(rgb(ON_CTA)).child("Текущий").into_any_element()
            } else {
                div().id(apply_id).px(px(6.)).py(px(2.)).rounded(px(R_SM)).bg(rgb(BG_INPUT)).font_family(FONT_PIXEL_ALT).text_size(px(9.)).font_weight(gpui::FontWeight::BOLD).text_color(rgb(TEXT_PRIMARY)).child("Надеть")
                    .on_click(cx.listener(move |this, _, _, cx| { this.upload_skin(bytes.clone()); cx.notify(); })).into_any_element()
            })
            .child(div().id(edit_id).px(px(4.)).py(px(2.)).rounded(px(R_SM)).bg(rgb(BG_INPUT)).font_family(FONT_PIXEL_ALT).text_size(px(9.)).text_color(rgb(TEXT_MUTED)).child("✏️")
                .on_click(cx.listener(move |this, _, _, cx| { prompt_rename_preset(this, edit_preset_id.clone(), name.clone(), cx); })))
            .child(div().id(del_id).px(px(4.)).py(px(2.)).rounded(px(R_SM)).bg(rgb(BG_INPUT)).font_family(FONT_PIXEL_ALT).text_size(px(9.)).text_color(rgb(TEXT_MUTED)).child("✕")
                .on_click(cx.listener(move |this, _, _, cx| { this.custom_presets.retain(|p| p.id != id); cx.notify(); }))))
        .into_any_element()
}

fn standard_preset_card(ui: &LauncherUI, name: &'static str, id: &'static str, cx: &mut Cx) -> AnyElement {
    let is_active = is_preset_active(ui, id);
    let border_clr = if is_active { CTA_HOV } else { BORDER };

    let img_el = if let Some(loaded_img) = ui.preset_images.get(id) {
        img(loaded_img.clone()).w(px(72.)).h(px(85.)).object_fit(gpui::ObjectFit::Contain).into_any_element()
    } else {
        div().w(px(72.)).h(px(85.)).flex().items_center().justify_center().child(ic("user", 24., if is_active { CTA } else { TEXT_MUTED })).into_any_element()
    };

    div().id(id).w(px(112.)).p(px(6.)).bg(rgb(BG_CARD)).rounded(px(R_SM)).border_1().border_color(rgb(border_clr)).hover(|s| s.bg(rgb(BG_INPUT))).cursor_pointer().flex().flex_col().items_center().gap(px(4.))
        .child(img_el)
        .child(div().font_family(FONT_PIXEL_ALT).text_size(px(11.)).font_weight(gpui::FontWeight::BOLD).text_color(rgb(if is_active { CTA } else { TEXT_PRIMARY })).child(name))
        .child(if is_active {
            div().px(px(12.)).py(px(2.)).rounded(px(R_SM)).bg(rgb(CTA)).font_family(FONT_PIXEL_ALT).text_size(px(9.)).font_weight(gpui::FontWeight::BOLD).text_color(rgb(ON_CTA)).child("Текущий").into_any_element()
        } else {
            div().px(px(12.)).py(px(2.)).rounded(px(R_SM)).bg(rgb(BG_INPUT)).border_1().border_color(rgb(BORDER)).font_family(FONT_PIXEL_ALT).text_size(px(10.)).font_weight(gpui::FontWeight::BOLD).text_color(rgb(TEXT_PRIMARY)).child("Надеть").into_any_element()
        })
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| apply_preset(this, id, cx)))
        .into_any_element()
}

fn apply_preset(this: &mut LauncherUI, name: &'static str, cx: &mut Cx) {
    let master_url = this.config.master_url.clone();
    let url = format!("{}/api/textures/presets/{}.png", master_url.trim_end_matches('/'), name);

    cx.spawn(async move |this, cx| {
        let loaded = crate::image_loader::load_image_and_bytes(url).await;
        let _ = this.update(cx, |this, cx| {
            if let Ok((_, bytes)) = loaded { this.upload_skin(bytes); }
            cx.notify();
        });
    }).detach();
}

fn is_grabbable(ui: &LauncherUI) -> bool {
    ui.skin_bytes.is_some() && ui.skin_preview.is_some()
}

fn preview_box(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    div().id("skin-preview-area").w_full().flex_1().min_h(px(PREVIEW_H)).bg(rgb(BG_INPUT)).rounded(px(R_SM)).border_1().border_color(rgb(BORDER)).overflow_hidden().flex().items_center().justify_center()
        .when(is_grabbable(ui), |d| d.cursor(CursorStyle::OpenHand).on_mouse_down(MouseButton::Left, cx.listener(skin_drag::on_grab)))
        .child(preview_content(ui))
        .into_any_element()
}

fn drag_hint() -> AnyElement {
    div().font_family(FONT_PIXEL_ALT).text_size(px(11.)).text_color(rgb(TEXT_MUTED)).child(t("profile-drag-to-rotate")).into_any_element()
}

fn preview_content(ui: &LauncherUI) -> AnyElement {
    if ui.skin_loading || ui.skin_uploading { return placeholder(t("profile-skin-loading")); }
    if let Some(p) = &ui.skin_preview { return img(p.clone()).w(px(PREVIEW_W)).h(px(PREVIEW_H)).into_any_element(); }
    if let Some(s) = &ui.skin_image { return img(s.clone()).w(px(PREVIEW_W)).h(px(PREVIEW_H)).into_any_element(); }
    placeholder(t("profile-no-skin"))
}

fn placeholder(text: impl Into<gpui::SharedString>) -> AnyElement {
    div().size_full().flex().items_center().justify_center().font_family(FONT_PIXEL_ALT).text_size(px(13.)).text_color(rgb(TEXT_MUTED)).child(text.into()).into_any_element()
}

fn prompt_rename_preset(this: &mut LauncherUI, preset_id: String, current_name: String, cx: &mut gpui::Context<LauncherUI>) {
    let script = format!(
        r#"text returned of (display dialog "Название пресета:" default answer "{}" with title "Переименование пресета")"#,
        current_name.replace('"', "\\\"")
    );
    if let Ok(output) = std::process::Command::new("osascript").arg("-e").arg(&script).output() {
        if output.status.success() {
            let new_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !new_name.is_empty() {
                if let Some(preset) = this.custom_presets.iter_mut().find(|p| p.id == preset_id) {
                    preset.name = new_name;
                }
                cx.notify();
            }
        }
    }
}

fn on_upload_click(this: &mut LauncherUI, _e: &gpui::ClickEvent, _w: &mut gpui::Window, cx: &mut gpui::Context<LauncherUI>) {
    let script = "POSIX path of (choose file of type {\"public.png\"} with prompt \"Select Minecraft skin PNG\")";
    if let Ok(output) = std::process::Command::new("osascript").arg("-e").arg(script).output() {
        if output.status.success() {
            let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !p.is_empty() {
                if let Ok(b) = std::fs::read(&p) {
                    if b.len() > 8 && &b[0..8] == b"\x89PNG\r\n\x1a\n" && b.len() < 256 * 1024 {
                        let filename = std::path::Path::new(&p)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Новый скин")
                            .to_string();
                        this.upload_skin(b.clone());
                        let preset = crate::state::SavedSkinPreset {
                            id: uuid::Uuid::new_v4().to_string(),
                            name: filename,
                            bytes: b,
                            preview: this.skin_preview.clone(),
                        };
                        this.custom_presets.push(preset);
                        cx.notify();
                        return;
                    }
                }
            }
        }
    }
    this.toast = Some(crate::state::Toast { text: t("profile-skin-invalid"), level: schema::NotifLevel::Warning });
    cx.notify();
}

fn on_import_by_username_click(this: &mut LauncherUI, _e: &gpui::ClickEvent, _w: &mut gpui::Window, cx: &mut gpui::Context<LauncherUI>) {
    let script = r#"text returned of (display dialog "Введите ник игрока Minecraft:" default answer "Dalynkaa" with title "Импорт скина по нику")"#;
    if let Ok(output) = std::process::Command::new("osascript").arg("-e").arg(script).output() {
        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !name.is_empty() {
                import_skin_by_username(this, name, cx);
            }
        }
    }
}

fn import_skin_by_username(this: &mut LauncherUI, username: String, cx: &mut gpui::Context<LauncherUI>) {
    let clean = username.trim().to_string();
    if clean.is_empty() { return; }
    let url = format!("https://minotar.net/skin/{}", clean);

    cx.spawn(async move |this, cx| {
        let loaded = crate::image_loader::load_image_and_bytes(url).await;
        let _ = this.update(cx, |this, cx| {
            if let Ok((_, bytes)) = loaded {
                this.upload_skin(bytes.clone());
                let preset = crate::state::SavedSkinPreset {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: clean,
                    bytes,
                    preview: this.skin_preview.clone(),
                };
                this.custom_presets.push(preset);
            } else {
                this.toast = Some(crate::state::Toast { text: "Не удалось скачать скин по нику".into(), level: schema::NotifLevel::Warning });
            }
            cx.notify();
        });
    }).detach();
}
