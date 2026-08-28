//! Sidebar: logo, server list, buttons along the bottom.
use super::common::Cx;
use super::sidebar_parts::{collapsed_logo_toggle, empty_hint, logo, nav_icon};
use super::sidebar_server::server_item;
use super::sidebar_user::user_card;
use crate::state::{LauncherUI, Page};
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, ClickEvent};

pub fn sidebar(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let collapsed = ui.sidebar_collapsed;
    let selected = ui.selected_server_id();
    let cards: Vec<AnyElement> = ui
        .servers
        .iter()
        .map(|s| server_item(ui, s, selected == Some(s.id), cx))
        .collect();

    div()
        .w(px(if collapsed { 76. } else { 280. }))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .bg(rgb(SIDEBAR))
        .border_r_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .h(px(72.))
                .px(px(16.))
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.))
                .border_b_1()
                .border_color(rgb(BORDER))
                .when(!collapsed, |d| {
                    d.child(logo(cx)).child(nav_icon(
                        "sidebar-toggle-btn",
                        "panel-left-close",
                        false,
                        cx.listener(|this, _e, _w, cx| {
                            this.sidebar_collapsed = true;
                            cx.notify();
                        }),
                    ))
                })
                .when(collapsed, |d| {
                    d.justify_center().child(collapsed_logo_toggle(cx))
                }),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .overflow_hidden()
                .px(px(8.))
                .py(px(8.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .children(cards)
                .when(ui.servers.is_empty() && !collapsed, |d| {
                    d.child(empty_hint())
                }),
        )
        .child(
            div()
                .h(px(64.))
                .border_t_1()
                .border_color(rgb(BORDER))
                .px(px(if collapsed { 4. } else { 12. }))
                .flex()
                .items_center()
                .when(collapsed, |d| {
                    d.justify_center()
                        .child(super::sidebar_user::user_avatar_only(ui, cx))
                })
                .when(!collapsed, |d| {
                    d.gap(px(4.))
                        .child(user_card(ui, cx))
                        .child(nav_icon(
                            "news-bottom",
                            "newspaper",
                            ui.page == Page::News,
                            cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                this.page = Page::News;
                                cx.notify();
                            }),
                        ))
                        .child(nav_icon(
                            "settings-bottom",
                            "settings",
                            ui.page == Page::Settings,
                            cx.listener(|this, _e: &ClickEvent, _w, cx| {
                                this.page = Page::Settings;
                                cx.notify();
                            }),
                        ))
                }),
        )
        .into_any_element()
}
