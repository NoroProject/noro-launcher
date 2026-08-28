//! Cape selection grid panel in profile (matching presets grid layout).

use super::common::{panel, Cx};
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::MessageToBackend;
use gpui::{div, img, prelude::*, px, rgb, AnyElement, FontWeight, SharedString};
use i18n::t;

pub fn cape_panel(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let user = ui.user.as_ref().expect("user in profile");
    let current_url = user.cape_url.as_deref();

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
                .id("capes-scroll")
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
                        .child(no_cape_card(current_url.is_none(), cx))
                        .children(ui.capes.iter().map(|cape| {
                            let is_sel = current_url == Some(cape.url.as_str());
                            cape_tile_card(ui, cape, is_sel, cx)
                        })),
                ),
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
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(t("profile-cape-title")),
        )
        .into_any_element()
}

fn no_cape_card(selected: bool, cx: &mut Cx) -> AnyElement {
    let border_clr = if selected { CTA_HOV } else { BORDER };

    div()
        .id("no-cape-tile")
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
        .on_click(cx.listener(|this, _, _, cx| {
            this.backend
                .send(MessageToBackend::SelectCape { cape_id: None });
            cx.notify();
        }))
        .child(
            div()
                .w(px(72.))
                .h(px(96.))
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(6.))
                .child(ic("eye-off", 26., if selected { CTA } else { TEXT_MUTED }))
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(9.))
                        .text_color(rgb(TEXT_MUTED))
                        .child(t("profile-cape-remove")),
                ),
        )
        .child(
            div()
                .w_full()
                .px(px(4.))
                .truncate()
                .text_center()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(10.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(if selected { CTA } else { TEXT_PRIMARY }))
                .child(t("profile-cape-none")),
        )
        .child(if selected {
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
                .font_weight(FontWeight::BOLD)
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
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(TEXT_PRIMARY))
                .child(t("profile-cape-take-off"))
                .into_any_element()
        })
        .into_any_element()
}

fn cape_tile_card(
    ui: &LauncherUI,
    cape: &schema::CapeRow,
    selected: bool,
    cx: &mut Cx,
) -> AnyElement {
    let id = cape.id;
    let card_id: SharedString = format!("cape-card-{}", id).into();
    let border_clr = if selected { CTA_HOV } else { BORDER };

    let img_el = if let Some(loaded_img) = ui.cape_images.get(&id) {
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
            .child(ic("layers", 24., if selected { CTA } else { TEXT_MUTED }))
            .into_any_element()
    };

    div()
        .id(card_id)
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
            this.backend
                .send(MessageToBackend::SelectCape { cape_id: Some(id) });
            cx.notify();
        }))
        .child(img_el)
        .child(
            div()
                .w_full()
                .px(px(4.))
                .truncate()
                .text_center()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(10.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(if selected { CTA } else { TEXT_PRIMARY }))
                .child(cape.name.clone()),
        )
        .child(if selected {
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
                .font_weight(FontWeight::BOLD)
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
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(TEXT_PRIMARY))
                .child(t("profile-preset-wear"))
                .into_any_element()
        })
        .into_any_element()
}

/// Fallback empty modal overlay (deprecated)
pub fn cape_modal(_ui: &LauncherUI, _cx: &mut Cx) -> AnyElement {
    div().into_any_element()
}
