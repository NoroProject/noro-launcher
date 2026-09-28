//! Sidebar: logo, server list, buttons along the bottom.
use super::common::Cx;
use super::sidebar_parts::{collapsed_logo_toggle, dot_icon, empty_hint, logo, nav_icon};
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
                .id("sidebar-servers-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
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
        // Иконки и карточка профиля разными строками. В одну они не помещались:
        // на пять кнопок и имя в сайдбаре 280 px, и имя схлопывалось в две
        // точки — единственное, что человек там ищет.
        .child(
            div()
                .border_t_1()
                .border_color(rgb(BORDER))
                .px(px(if collapsed { 4. } else { 8. }))
                .py(px(8.))
                .flex()
                .flex_col()
                .gap(px(4.))
                // Свёрнутый сайдбар: колокольчик остаётся, иначе уведомления
                // из него недостижимы вовсе.
                .child(nav_row(ui, collapsed, cx))
                .when(!collapsed, |d| d.child(user_card(ui, cx)))
                .when(collapsed, |d| {
                    d.items_center()
                        .child(super::sidebar_user::user_avatar_only(ui, cx))
                }),
        )
        .into_any_element()
}

/// The row of section buttons above the profile card.
fn nav_row(ui: &LauncherUI, collapsed: bool, cx: &mut Cx) -> AnyElement {
    div()
        .flex()
        .when(collapsed, |d| d.flex_col().items_center())
        .when(!collapsed, |d| d.justify_between())
        .gap(px(2.))
        .child(super::notifications::bell(ui, cx))
        .child(dot_icon(
            "messages-bottom",
            "mail",
            ui.page == Page::Messages,
            super::messages::unread_total(ui) > 0,
            cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.page = Page::Messages;
                cx.notify();
            }),
        ))
        .child(nav_icon(
            "account-bottom",
            "shield",
            ui.page == Page::Account,
            cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.page = Page::Account;
                cx.notify();
            }),
        ))
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
        .into_any_element()
}
