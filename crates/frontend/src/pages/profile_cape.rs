//! Cape selection card and grid modal overlay in profile.

use super::common::{panel, Cx};
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::MessageToBackend;
use gpui::{div, img, prelude::*, px, rgb, rgba, AnyElement, FontWeight, SharedString};
use i18n::t;

pub fn cape_panel(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let user = ui.user.as_ref().expect("user in profile");
    let current_url = user.cape_url.as_deref();
    let current_cape = ui.capes.iter().find(|c| Some(c.url.as_str()) == current_url);

    let active_label: SharedString = match (current_url, current_cape) {
        (None, _) => SharedString::from("Отключен"),
        (Some(_), Some(cape)) => SharedString::from(cape.name.clone()),
        (Some(_), None) => SharedString::from("Активный плащ"),
    };

    panel().p(px(20.)).flex().items_center().justify_between().child(
        div().flex().items_center().gap(px(12.))
            .child(icon_box(current_url.is_some()))
            .child(text_block(active_label, current_url.is_some())),
    ).child(
        div().cursor_pointer().px(px(16.)).py(px(10.)).rounded(px(R_SM)).bg(rgb(BG_INPUT)).border_1().border_color(rgb(BORDER)).hover(|s| s.border_color(rgb(CTA))).flex().items_center().gap(px(8.))
            .child(div().font_family(FONT_PIXEL_ALT).text_size(px(12.)).font_weight(FontWeight::BOLD).text_color(rgb(TEXT_PRIMARY)).child("ВЫБРАТЬ ПЛАЩ"))
            .child(ic("layers", 16., TEXT_SECONDARY))
            .on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                this.cape_selector_open = true;
                if this.capes.is_empty() { this.backend.send(MessageToBackend::RequestCapesList); }
                cx.notify();
            })),
    ).into_any_element()
}

fn icon_box(active: bool) -> AnyElement {
    div().size(px(48.)).rounded(px(R_SM)).bg(rgb(BG_INPUT)).border_1().border_color(rgb(if active { CTA } else { BORDER })).flex().items_center().justify_center()
        .child(ic(if active { "image" } else { "eye-off" }, 20., if active { CTA } else { TEXT_MUTED })).into_any_element()
}

fn text_block(label: SharedString, active: bool) -> AnyElement {
    div().flex().flex_col().gap(px(2.))
        .child(div().font_family(FONT_PIXEL_ALT).text_size(px(14.)).font_weight(FontWeight::BOLD).text_color(rgb(TEXT_PRIMARY)).child(t("profile-cape").to_uppercase()))
        .child(div().font_family(FONT_PIXEL_ALT).text_size(px(12.)).text_color(rgb(if active { CTA } else { TEXT_MUTED })).child(label)).into_any_element()
}

/// Grid Modal Overlay for selecting capes
pub fn cape_modal(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let user = ui.user.as_ref().expect("user in profile");
    let current_url = user.cape_url.as_deref();

    div().absolute().inset_0().bg(rgba(0x081020ee)).flex().items_center().justify_center().child(
        div().w(px(640.)).max_h(px(520.)).bg(rgb(BG_PANEL)).border_1().border_color(rgb(BORDER)).rounded(px(R_MD)).p(px(24.)).flex().flex_col().gap(px(16.))
            .child(
                div().flex().items_center().justify_between()
                    .child(div().font_family(FONT_PIXEL).text_size(px(16.)).text_color(rgb(CTA)).child("ВЫБОР ПЛАЩА"))
                    .child(div().cursor_pointer().p(px(4.)).child(ic("x", 18., TEXT_MUTED)).on_mouse_down(gpui::MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.cape_selector_open = false; cx.notify();
                    }))),
            )
            .child(
                div().id("cape-modal-scroll").flex_1().min_h_0().overflow_y_scroll().flex().flex_wrap().gap(px(12.))
                    .child(no_cape_card(current_url.is_none(), cx.listener(|this, _, _, cx| {
                        this.cape_selector_open = false;
                        this.backend.send(MessageToBackend::SelectCape { cape_id: None });
                        cx.notify();
                    })))
                    .children(ui.capes.iter().map(|cape| {
                        let is_sel = current_url == Some(cape.url.as_str());
                        let id = cape.id;
                        cape_card(ui, cape, is_sel, cx.listener(move |this, _, _, cx| {
                            this.cape_selector_open = false;
                            this.backend.send(MessageToBackend::SelectCape { cape_id: Some(id) });
                            cx.notify();
                        }))
                    })),
            ),
    ).into_any_element()
}

fn no_cape_card(selected: bool, on_click: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static) -> AnyElement {
    div().cursor_pointer().w(px(104.)).h(px(150.)).rounded(px(R_SM)).bg(rgb(if selected { BG_INPUT } else { BG_CARD }))
        .border_1().border_color(rgb(if selected { CTA } else { BORDER })).hover(|s| s.bg(rgb(BG_INPUT)).border_color(rgb(CTA)))
        .flex().flex_col().items_center().justify_center().gap(px(8.))
        .child(ic("x", 24., if selected { CTA } else { TEXT_MUTED }))
        .child(div().font_family(FONT_PIXEL_ALT).text_size(px(11.)).font_weight(FontWeight::BOLD).text_color(rgb(if selected { CTA } else { TEXT_MUTED })).child("Без плаща"))
        .on_mouse_down(gpui::MouseButton::Left, on_click).into_any_element()
}

fn cape_card(ui: &LauncherUI, cape: &schema::CapeRow, selected: bool, on_click: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static) -> AnyElement {
    let img_content = if let Some(loaded_img) = ui.cape_images.get(&cape.id) {
        img(loaded_img.clone()).size_full().object_fit(gpui::ObjectFit::Contain).into_any_element()
    } else {
        ic("layers", 24., CTA).into_any_element()
    };

    div().cursor_pointer().w(px(104.)).h(px(150.)).p(px(6.)).rounded(px(R_SM)).bg(rgb(if selected { BG_INPUT } else { BG_CARD }))
        .border_1().border_color(rgb(if selected { CTA } else { BORDER })).hover(|s| s.bg(rgb(BG_INPUT)).border_color(rgb(CTA)))
        .flex().flex_col().items_center().justify_between()
        .child(div().w_full().truncate().font_family(FONT_PIXEL_ALT).text_size(px(11.)).font_weight(FontWeight::BOLD).text_color(rgb(if selected { CTA } else { TEXT_PRIMARY })).child(cape.name.clone()))
        .child(div().w(px(64.)).h(px(100.)).flex().items_center().justify_center().child(img_content))
        .child(if selected {
            div().px(px(6.)).py(px(2.)).rounded(px(R_SM)).bg(rgb(CTA)).font_family(FONT_PIXEL_ALT).text_size(px(9.)).font_weight(FontWeight::BOLD).text_color(rgb(ON_CTA)).child("АКТИВЕН")
        } else { div() })
        .on_mouse_down(gpui::MouseButton::Left, on_click).into_any_element()
}
