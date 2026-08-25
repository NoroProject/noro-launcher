//! Вкладка MODS — список опциональных модов сервера в полностраничном режиме.
use super::common::{tabs, Cx};
use super::mod_icon::{category_color, mod_icon, mod_text};
use crate::components::{badge, mod_toggle};
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::OptionalModInfo;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use i18n::t;
use uuid::Uuid;

pub fn page(ui: &mut LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let mods = ui
        .optional_mods
        .get(&server_id)
        .cloned()
        .unwrap_or_default();
    for icon_url in mods.iter().filter_map(|m| m.icon_url.clone()) {
        ui.ensure_optional_mod_icon_loaded(Some(icon_url), cx);
    }

    div()
        .size_full()
        .relative()
        .bg(rgb(CONTENT_FALLBACK))
        .child(tabs(ui, cx))
        .child(
            div()
                .absolute()
                .top(px(104.))
                .left(px(32.))
                .right(px(32.))
                .bottom(px(32.))
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(page_header(ui, server_id, &mods, cx))
                .child(mod_list(ui, server_id, &mods, cx)),
        )
        .into_any_element()
}

fn page_header(
    ui: &LauncherUI,
    server_id: Uuid,
    mods: &[OptionalModInfo],
    cx: &mut Cx,
) -> AnyElement {
    let enabled = mods.iter().filter(|m| m.enabled).count();
    let allow_suggest = ui
        .allow_mod_suggestions
        .get(&server_id)
        .copied()
        .unwrap_or(true);

    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(ic("package-plus", 20., ACCENT))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(18.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(t("mods-optional")),
        )
        .child(div().flex_1())
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_MUTED))
                .child(format!("{enabled} / {} active", mods.len())),
        )
        .when(allow_suggest, |d| {
            d.child(crate::components::btn(
                "mods-suggest-btn",
                "+ Suggest Mod",
                false,
                cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    this.page = crate::state::Page::ServerModCatalog(server_id);
                    cx.notify();
                }),
            ))
        })
        .into_any_element()
}

fn mod_list(ui: &LauncherUI, server_id: Uuid, mods: &[OptionalModInfo], cx: &mut Cx) -> AnyElement {
    if mods.is_empty() {
        return div()
            .p(px(32.))
            .rounded(px(R_MD))
            .bg(rgb(BG_PANEL))
            .border_1()
            .border_color(rgb(BORDER))
            .flex()
            .items_center()
            .justify_center()
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(14.))
            .text_color(rgb(TEXT_MUTED))
            .child(t("mods-empty"))
            .into_any_element();
    }

    let rows: Vec<AnyElement> = mods.iter().map(|m| mod_row(ui, server_id, m, cx)).collect();
    div()
        .rounded(px(R_MD))
        .bg(rgb(BG_PANEL))
        .border_1()
        .border_color(rgb(BORDER))
        .overflow_hidden()
        .flex()
        .flex_col()
        .children(rows)
        .into_any_element()
}

fn mod_row(ui: &LauncherUI, server_id: Uuid, m: &OptionalModInfo, cx: &mut Cx) -> AnyElement {
    let name = m.name.clone();
    let color = category_color(&m.category);

    div()
        .h(px(72.))
        .px(px(20.))
        .border_b_1()
        .border_color(rgb(BORDER))
        .flex()
        .items_center()
        .gap(px(16.))
        .hover(|d| d.bg(rgba(0xffffff0a)))
        .child(mod_icon(ui, m, color))
        .child(mod_text(m, None, 60))
        .child(div().flex_1())
        // Мод с ограничением: его ставит не каждый, а тот, кому выдали право.
        // Раньше метка называлась «VIP» — от старой затеи с донатом; к правам
        // это отношения не имеет и путало.
        .when(m.limited, |d| d.child(badge(t("mods-limited"), WARNING)))
        .child(badge(m.category.clone(), color))
        .child(mod_toggle(
            SharedString::from(format!("mods-tgl-{server_id}-{}", m.name)),
            m.enabled,
            m.allowed,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                this.toggle_optional(server_id, &name);
                cx.notify();
            }),
        ))
        .into_any_element()
}
