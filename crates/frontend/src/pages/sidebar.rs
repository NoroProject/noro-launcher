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
    let selected = ui.selected_server_id();
    let cards: Vec<AnyElement> = ui
        .servers
        .iter()
        .map(|s| server_item(ui, s, selected == Some(s.id), cx))
        .collect();

    div()
        .w(px(280.))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .bg(rgb(SIDEBAR))
        .border_r_1()
        .border_color(rgb(BORDER))
        // Логотип
        .child(
            div()
                .h(px(80.))
                .px(px(20.))
                .flex()
                .items_center()
                .gap(px(12.))
                .border_b_1()
                .border_color(rgb(BORDER))
                .child(logo(cx))
                .child(div().flex_1())
                .child(
                    div()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(9.))
                        .text_color(rgb(TEXT_MUTED))
                        .child(t(if ui.online {
                            "sidebar-online"
                        } else {
                            "sidebar-offline"
                        })),
                ),
        )
        // Список серверов (высота по содержимому)
        .child(
            div()
                .px(px(8.))
                .py(px(8.))
                .flex()
                .flex_col()
                .gap(px(4.))
                .children(cards)
                .when(ui.servers.is_empty(), |d| d.child(empty_hint())),
        )
        // Прозрачный spacer — отодвигает нижнюю панель вниз
        .child(div().flex_1())
        // Нижняя панель
        .child(
            div()
                .border_t_1()
                .border_color(rgb(BORDER))
                .px(px(12.))
                .py(px(10.))
                .flex()
                .gap(px(4.))
                .items_center()
                .child(user_card(ui, cx))
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
        )
        .into_any_element()
}
