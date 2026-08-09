//! Экран поиска и запроса модов в лаунчере для игроков.
use super::common::{tabs, Cx};
use crate::components::btn;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::MessageToBackend;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use uuid::Uuid;

pub fn page(ui: &mut LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
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
                .child(page_header(server_id, cx))
                .child(mod_catalog_body(ui, server_id, cx)),
        )
        .into_any_element()
}

fn page_header(server_id: Uuid, cx: &mut Cx) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(ic("search", 20., ACCENT))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(18.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child("Suggest Optional Mod"),
        )
        .child(div().flex_1())
        .child(btn(
            "back-to-mods-btn",
            "Back to Mods",
            false,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                this.page = crate::state::Page::ServerMods(server_id);
                cx.notify();
            }),
        ))
        .into_any_element()
}

fn mod_catalog_body(_ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    div()
        .rounded(px(R_MD))
        .bg(rgb(BG_PANEL))
        .border_1()
        .border_color(rgb(BORDER))
        .p(px(24.))
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(14.))
                .text_color(rgb(TEXT_PRIMARY))
                .child("Find mods from Modrinth or CurseForge to request addition to this server assembly:"),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(suggest_mod_preset(
                    server_id,
                    "modrinth",
                    "sodium",
                    "Sodium",
                    "Modern rendering engine for Minecraft",
                    cx,
                ))
                .child(suggest_mod_preset(
                    server_id,
                    "modrinth",
                    "iris",
                    "Iris Shaders",
                    "Modern shader pack loader for Minecraft",
                    cx,
                ))
                .child(suggest_mod_preset(
                    server_id,
                    "modrinth",
                    "xaeros-minimap",
                    "Xaero's Minimap",
                    "Displays a mini-map in the corner of your screen",
                    cx,
                ))
                .child(suggest_mod_preset(
                    server_id,
                    "modrinth",
                    "appleskin",
                    "AppleSkin",
                    "Food and hunger HUD improvements",
                    cx,
                )),
        )
        .into_any_element()
}

fn suggest_mod_preset(
    server_id: Uuid,
    provider: &'static str,
    project_id: &'static str,
    title: &'static str,
    description: &'static str,
    cx: &mut Cx,
) -> AnyElement {
    let provider_str = provider.to_string();
    let proj_id_str = project_id.to_string();
    let title_str = title.to_string();
    let desc_str = description.to_string();

    div()
        .h(px(56.))
        .px(px(16.))
        .rounded(px(R_SM))
        .bg(rgba(0xffffff0a))
        .border_1()
        .border_color(rgb(BORDER))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(12.))
        .child(
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(14.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(TEXT_PRIMARY))
                        .child(title),
                )
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(11.))
                        .text_color(rgb(TEXT_MUTED))
                        .child(description),
                ),
        )
        .child(btn(
            SharedString::from(format!("btn-sug-{project_id}")),
            "Request Mod",
            true,
            cx.listener(move |this, _e: &ClickEvent, _w, _cx| {
                this.backend.send(MessageToBackend::SuggestOptionalMod {
                    server_id,
                    build_id: None,
                    provider: provider_str.clone(),
                    project_id: proj_id_str.clone(),
                    title: title_str.clone(),
                    icon_url: None,
                    description: Some(desc_str.clone()),
                });
            }),
        ))
        .into_any_element()
}
