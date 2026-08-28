use super::common::{tabs, Cx};
use super::game_empty::empty;
use super::{game_bar, game_status, game_sync};
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{
    div, img, linear_color_stop, linear_gradient, prelude::*, px, rgb, rgba, AnyElement, ObjectFit,
};
use schema::ServerEntry;

pub fn page(ui: &mut LauncherUI, cx: &mut Cx) -> AnyElement {
    let Some(server) = ui
        .selected_server_id()
        .and_then(|id| ui.server(&id).cloned())
    else {
        return empty(ui, cx);
    };

    ui.ensure_background_loaded(server.id, server.background_url.clone(), cx);
    let sync = ui.sync_state(&server.id);
    let locked = server.limited
        && !ui
            .user
            .as_ref()
            .map(|u| u.can_join_server(&server.id, true))
            .unwrap_or(false);

    div()
        .size_full()
        .relative()
        .overflow_hidden()
        .bg(rgb(CONTENT_FALLBACK))
        .child(background(ui, &server))
        .child(tabs(ui, cx))
        .child(game_status::info_block(&server))
        .when(sync.syncing || sync.failed.is_some(), |d| {
            d.child(game_sync::sync_overlay(server.id, &sync, cx))
        })
        .child(game_bar::bottom_bar(ui, &server, &sync, locked, cx))
        .into_any_element()
}

fn background(ui: &LauncherUI, server: &ServerEntry) -> AnyElement {
    let image = ui.background_images.get(&server.id).cloned();
    let has_bg = ui.background_images.contains_key(&server.id);

    let base = div()
        .absolute()
        .top(px(0.))
        .left(px(0.))
        .right(px(0.))
        .bottom(px(0.))
        .bg(rgb(CONTENT_FALLBACK));

    let with_content = if let Some(img_data) = image {
        base.child(img(img_data).size_full().object_fit(ObjectFit::Cover))
    } else if !has_bg {
        base.child(placeholder(server))
    } else {
        base
    };

    with_content
        .child(
            div()
                .absolute()
                .top(px(0.))
                .left(px(0.))
                .right(px(0.))
                .bottom(px(0.))
                .bg(rgba(0x08102074)),
        )
        .child(bottom_fade())
        .into_any_element()
}

fn bottom_fade() -> AnyElement {
    div()
        .absolute()
        .left(px(0.))
        .right(px(0.))
        .bottom(px(0.))
        .h(px(360.))
        .bg(linear_gradient(
            180.,
            linear_color_stop(rgba(0x0a162600), 0.0),
            linear_color_stop(rgba(0x0a1626f2), 1.0),
        ))
        .into_any_element()
}

fn placeholder(server: &ServerEntry) -> AnyElement {
    let initial: String = server
        .name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".into());
    let hue = (server.id.as_u128() % 6) as usize;
    let color = [ACCENT, BLUE, SUCCESS, WARNING, 0xdbb2ff, CTA][hue];

    div()
        .absolute()
        .top(px(0.))
        .left(px(0.))
        .right(px(0.))
        .bottom(px(0.))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .font_family(FONT_PIXEL)
                .text_size(px(180.))
                .text_color(rgba((color << 8) | 0x0e))
                .child(initial),
        )
        .into_any_element()
}
