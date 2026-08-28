//! Server card in the sidebar: icon (real or initial), status dot, version.
use super::common::Cx;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{
    div, img, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, ObjectFit,
    SharedString,
};
use schema::ServerEntry;

pub fn server_item(
    ui: &LauncherUI,
    server: &ServerEntry,
    selected: bool,
    cx: &mut Cx,
) -> AnyElement {
    let id = server.id;
    let locked = server.limited
        && !ui
            .user
            .as_ref()
            .map(|u| u.can_join_server(&id, true))
            .unwrap_or(false);
    let version = server
        .current_version
        .clone()
        .unwrap_or_else(|| "—".to_string());
    let sync = ui.sync_state(&id);

    let collapsed = ui.sidebar_collapsed;
    div()
        .id(SharedString::from(format!("server-nav-{id}")))
        .h(px(56.))
        .w_full()
        .flex()
        .items_center()
        .justify_center()
        .gap(px(12.))
        .px(px(if collapsed { 4. } else { 12. }))
        .rounded(px(R_MD))
        .cursor_pointer()
        .bg(if selected {
            rgba(0x1f355666)
        } else {
            rgba(0x00000000)
        })
        .when(selected, |d| {
            d.border_1().border_color(rgba((CTA << 8) | 0x60))
        })
        .hover(|d| d.bg(rgba(0x1f355640)))
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.open_server(id);
            cx.notify();
        }))
        .child(icon_slot(ui, server))
        .when(!collapsed, |d| d.child(info_block(server, &version)))
        .when(!collapsed, |d| d.child(div().flex_1()))
        .when(locked && !collapsed, |d| d.child(ic("lock", 14., WARNING)))
        .when(!collapsed, |d| d.child(status_dot(&sync)))
        .into_any_element()
}

fn icon_slot(ui: &LauncherUI, server: &ServerEntry) -> AnyElement {
    let icon_image = ui.server_icons.get(&server.id).cloned();
    let outer = div()
        .size(px(40.))
        .rounded(px(R_SM))
        .overflow_hidden()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center();

    if let Some(img_data) = icon_image {
        outer
            .child(img(img_data).size_full().object_fit(ObjectFit::Cover))
            .into_any_element()
    } else {
        // A coloured initial, while the icon loads or if there is none.
        let initial: SharedString = server
            .name
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".into())
            .into();
        let hue = (server.id.as_u128() % 6) as u32;
        let color = [ACCENT, BLUE, SUCCESS, WARNING, ERROR, CTA][hue as usize];
        outer
            .bg(rgba((color << 8) | 0x22))
            .border_1()
            .border_color(rgba((color << 8) | 0x44))
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(14.))
            .font_weight(FontWeight::BOLD)
            .text_color(rgb(color))
            .child(initial)
            .into_any_element()
    }
}

fn info_block(server: &ServerEntry, version: &str) -> AnyElement {
    div()
        .min_w_0()
        .flex_col()
        .flex()
        .gap(px(3.))
        .child(
            div()
                .truncate()
                .font_family(FONT_PIXEL_ALT)
                .font_weight(FontWeight::BOLD)
                .text_size(px(13.))
                .text_color(rgb(TEXT_PRIMARY))
                .child(server.name.clone()),
        )
        .child(
            div()
                .truncate()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .text_color(rgb(TEXT_MUTED))
                .child(format!("{} · v{}", server.mc_version, version)),
        )
        .into_any_element()
}

fn status_dot(sync: &crate::state::SyncUiState) -> AnyElement {
    let color = if sync.running {
        SUCCESS
    } else if sync.syncing {
        WARNING
    } else if sync.failed.is_some() {
        ERROR
    } else {
        BORDER
    };
    div()
        .size(px(8.))
        .rounded_full()
        .flex_shrink_0()
        .bg(rgb(color))
        .into_any_element()
}
