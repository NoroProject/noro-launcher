//! Боковая панель: логотип, список серверов, кнопки внизу.
use super::common::Cx;
use super::sidebar_parts::{empty_hint, logo, nav_icon};
use super::sidebar_server::server_item;
use super::sidebar_user::user_card;
use crate::state::{LauncherUI, Page};
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement};
use i18n::t;

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
        // Логотип + Кнопка сворачивания
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
                .when(!collapsed, |d| d.child(logo(cx)))
                .child(nav_icon(
                    "sidebar-toggle-btn",
                    "menu",
                    collapsed,
                    cx.listener(|this, _e, _w, cx| {
                        this.sidebar_collapsed = !this.sidebar_collapsed;
                        cx.notify();
                    }),
                )),
        )
        // Список серверов
        .child(
            div()
                .px(px(8.))
                .py(px(8.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .children(cards)
                .when(ui.servers.is_empty() && !collapsed, |d| d.child(empty_hint())),
        )
        .child(div().flex_1())
        // Нижняя панель
        .child(
            div()
                .border_t_1()
                .border_color(rgb(BORDER))
                .px(px(8.))
                .py(px(10.))
                .flex()
                .flex_col()
                .gap(px(6.))
                .items_center()
                .when(!collapsed, |d| d.child(user_card(ui, cx)))
                .child(
                    div()
                        .flex()
                        .gap(px(4.))
                        .items_center()
                        .child(nav_icon(
                            "news-bottom",
                            "newspaper",
                            ui.page == Page::News,
                            cx.listener(|this, _e, _w, cx| {
                                this.page = Page::News;
                                cx.notify();
                            }),
                        ))
                        .child(nav_icon(
                            "settings-bottom",
                            "settings",
                            ui.page == Page::Settings,
                            cx.listener(|this, _e, _w, cx| {
                                this.page = Page::Settings;
                                cx.notify();
                            }),
                        )),
                ),
        )
        .into_any_element()
}
