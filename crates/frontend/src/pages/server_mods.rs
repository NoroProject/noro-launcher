//! MODS tab: the server's optional mods as a full page.
use super::common::{tabs, Cx};
use super::mod_icon::{category_color, mod_icon, mod_text};
use crate::components::mod_toggle;
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
        .child({
            let mut args = i18n::FluentArgs::new();
            args.set("on", enabled.to_string());
            args.set("total", mods.len().to_string());
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_MUTED))
                .child(i18n::t_args("mods-active-count", &args))
        })
        // Not a button across the caption but a quiet action next to the counter:
        // suggesting a mod is a rare step and has no business competing with the
        // list of mods itself.
        .when(allow_suggest, |d| {
            d.child(
                div()
                    .id("mods-suggest-btn")
                    .h(px(32.))
                    .px(px(12.))
                    .rounded(px(R_SM))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .cursor_pointer()
                    .bg(rgba(0xffffff0a))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .hover(|s| s.bg(rgba(0xffffff14)).border_color(rgb(ACCENT)))
                    .child(ic("send", 13., ACCENT))
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(rgb(TEXT_SECONDARY))
                            .child(t("content-mode-suggest")),
                    )
                    .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                        // The mode comes from the way in, not a toggle on screen: people come
                        // from here to suggest, and from the "Mods" tab to install.
                        this.content_mode = crate::state::ContentMode::Suggest;
                        this.page = crate::state::Page::ServerModCatalog(server_id);
                        cx.notify();
                    })),
            )
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
        .h(px(60.))
        .px(px(16.))
        .border_b_1()
        .border_color(rgb(BORDER))
        .flex()
        .items_center()
        .gap(px(12.))
        .hover(|d| d.bg(rgba(0xffffff0a)))
        .child(mod_icon(ui, m, color))
        .child(mod_text(m, None, 60))
        .child(div().flex_1())
        // The category is a caption, not a badge. A badge next to the "by right"
        // badge and the toggle turned the right edge of the row into a set of
        // coloured buttons of which exactly one can be pressed.
        .child(
            div()
                .flex_shrink_0()
                .text_size(px(11.))
                .text_color(rgb(color))
                .child(m.category.clone()),
        )
        // A mod granted by a role: an icon instead of words. Few people see it, yet
        // it took room from everyone.
        .when(m.limited, |d| {
            d.child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .child(ic("lock", 12., WARNING))
                    .child(
                        div()
                            .text_size(px(10.))
                            .text_color(rgb(WARNING))
                            .child(t("mods-limited")),
                    ),
            )
        })
        .child(mod_toggle(
            SharedString::from(format!("mods-tgl-{server_id}-{}", m.name)),
            m.enabled,
            m.allowed,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                this.toggle_optional(server_id, &name, cx);
                cx.notify();
            }),
        ))
        .into_any_element()
}
