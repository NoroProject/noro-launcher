use super::{
    game, impersonate_dialog, log_request_dialog, news, news_detail, profile, remote_action_dialog,
    server_mod_catalog, server_mods, server_settings, settings, sidebar,
};
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
    // Отметка о чужом аккаунте живёт в рамке окна — см. `window_chrome`.
    let dialog = impersonate_dialog::dialog(ui, cx);
    let log_dialog = log_request_dialog::dialog(ui, cx);
    let remote_dialog = remote_action_dialog::dialog(ui, cx);

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(rgb(BG_WINDOW))
        .child(
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .child(sidebar::sidebar(ui, cx))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .relative()
                        .child(match ui.page.clone() {
                            Page::News => news::page(ui, cx),
                            Page::NewsDetail(id) => news_detail::page(ui, id, cx),
                            Page::Profile => profile::page(ui, cx),
                            Page::Settings => settings::page(ui, cx),
                            Page::ServerMods(id) => server_mods::page(ui, id, cx),
                            Page::ServerModCatalog(id) => server_mod_catalog::page(ui, id, cx),
                            Page::ServerSettings(id) => server_settings::page(ui, id, cx),
                            Page::Login | Page::Servers | Page::ServerDetail(_) => {
                                game::page(ui, cx)
                            }
                        })
                        .children(dialog)
                        .children(log_dialog)
                        .children(remote_dialog),
                ),
        )
        .into_any_element()
}
