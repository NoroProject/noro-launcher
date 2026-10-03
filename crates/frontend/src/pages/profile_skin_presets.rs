// Over 150 lines: the grid, the built-in preset cards and the add tile; the
// cards and the tile only appear in the grid.
//! The preset grid next to the skin preview: the built-in skins and the
//! add-a-preset tile.

use super::common::{panel, Cx};
use super::profile_skin_custom::custom_preset_card;
use super::profile_skin_pick::on_upload_click;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, img, prelude::*, px, rgb, rgba, AnyElement, ClickEvent};
use i18n::t;

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
