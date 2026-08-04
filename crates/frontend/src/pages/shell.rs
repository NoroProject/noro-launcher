use super::{game, news, profile, server_mods, server_settings, settings, sidebar};
use crate::state::{LauncherUI, Page};
use crate::theme::*;
use gpui::{div, prelude::*, rgb, AnyElement};

pub fn launcher_shell(ui: &mut LauncherUI, cx: &mut super::common::Cx) -> AnyElement {
    let to_load: Vec<_> = ui
        .servers
        .iter()
        .map(|s| (s.id, s.icon_url.clone()))
        .collect();
    for (id, url) in to_load {
        ui.ensure_icon_loaded(id, url, cx);
    }
    div()
        .size_full()
        .flex()
        .bg(rgb(BG_WINDOW))
        .child(sidebar::sidebar(ui, cx))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .h_full()
                .relative()
                .child(match ui.page.clone() {
                    Page::News => news::page(ui, cx),
                    Page::Profile => profile::page(ui, cx),
                    Page::Settings => settings::page(ui, cx),
                    Page::ServerMods(id) => server_mods::page(ui, id, cx),
                    Page::ServerSettings(id) => server_settings::page(ui, id, cx),
                    Page::Login | Page::Servers | Page::ServerDetail(_) => game::page(ui, cx),
                }),
        )
        .into_any_element()
}
