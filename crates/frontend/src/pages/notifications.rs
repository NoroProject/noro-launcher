// Over 150 lines: the bell, the panel and one card are one component — a card
// on its own has nowhere to say what happens when it is clicked.
//! The notification centre: a bell in the sidebar and the feed behind it.
//!
//! A panel rather than a page. Everything in the feed points somewhere else —
//! a case, a ticket, a build — so it has to be readable without losing what is
//! on screen, the way it is on the site.
//!
//! The text arrives twice: already translated by the master, and as the key it
//! came from. The key wins whenever the launcher's own catalogue has it, since
//! the master translates into its own language and the player picked theirs
//! here.

use super::common::Cx;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::MessageToBackend;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight};
use i18n::t;
use schema::notifications::{Level, Notification};

/// The bell, with an unread count when there is one.
pub fn bell(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let unread = ui.unread;
    let open = ui.notifications_open;

    div()
        .id("notifications-bell")
        .relative()
        .size(px(36.))
        .flex_shrink_0()
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .bg(if open {
            rgba((CTA << 8) | 0x18)
        } else {
            rgba(0x00000000)
        })
        .hover(|d| d.bg(rgba(0xffffff10)))
        .child(ic(
            if unread > 0 { "bell-dot" } else { "bell" },
            16.,
            if open {
                CTA
            } else if unread > 0 {
                ACCENT
            } else {
                TEXT_MUTED
            },
        ))
        .when(unread > 0, |d| d.child(badge(unread)))
        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
            this.notifications_open = !this.notifications_open;
            if this.notifications_open {
                this.notifications_loading = true;
                this.backend.send(MessageToBackend::RequestNotifications {
                    offset: 0,
                    unread_only: this.notifications_unread_only,
                });
            }
            cx.notify();
        }))
        .into_any_element()
}

/// Past 99 the exact number stops being information and starts being a wide
/// badge.
fn badge(unread: i64) -> AnyElement {
    let text = if unread > 99 {
        "99+".to_string()
    } else {
        unread.to_string()
    };
    div()
        .absolute()
        .top(px(2.))
        .right(px(2.))
        .min_w(px(16.))
        .h(px(16.))
        .px(px(4.))
        .rounded(px(8.))
        .bg(rgb(ACCENT))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(9.))
                .text_color(rgb(BG_WINDOW))
                .child(text),
        )
        .into_any_element()
}

/// The feed itself. Returns nothing while the bell is closed, so the caller can
/// hand it to `children` unconditionally.
pub fn panel(ui: &LauncherUI, cx: &mut Cx) -> Option<AnyElement> {
    if !ui.notifications_open {
        return None;
    }

    let cards: Vec<AnyElement> = ui.notifications.iter().map(|n| card(n, cx)).collect();

    Some(
        // A backdrop that closes on click: without one, the only way out is the
        // bell, and a panel that traps you is worse than no panel.
        div()
            .id("notifications-backdrop")
            .absolute()
            .inset_0()
            .bg(rgba(0x00000055))
            .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.notifications_open = false;
                cx.notify();
            }))
            .child(
                div()
                    .id("notifications-panel")
                    .occlude()
                    .on_click(|_, _, _| {})
                    .absolute()
                    .left(px(16.))
                    .bottom(px(16.))
                    // Высота по содержимому: фиксированная растягивала панель
                    // от колокольчика до верха окна ради одной строки «пусто».
                    .w(px(420.))
                    .max_h(px(520.))
                    .flex()
                    .flex_col()
                    .rounded(px(R_LG))
                    .bg(rgb(BG_PANEL))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .child(header(ui, cx))
                    .child(
                        div()
                            .id("notifications-scroll")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .p(px(8.))
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .children(cards)
                            .when(ui.notifications.is_empty(), |d| d.child(empty(ui))),
                    ),
            )
            .into_any_element(),
    )
}

fn header(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let unread_only = ui.notifications_unread_only;

    div()
        .h(px(42.))
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(6.))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .flex_1()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(t("launcher-notifications")),
        )
        .child(filter_toggle(unread_only, cx))
        .child(
            div()
                .id("notifications-read-all")
                .size(px(28.))
                .rounded(px(R_SM))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|d| d.bg(rgba(0xffffff10)))
                .child(ic("check-check", 15., TEXT_MUTED))
                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.backend
                        .send(MessageToBackend::MarkAllNotificationsRead);
                    cx.notify();
                })),
        )
        .into_any_element()
}

fn filter_toggle(unread_only: bool, cx: &mut Cx) -> AnyElement {
    crate::components::segment(
        "notifications-filter",
        None,
        t("launcher-notifications-unread-only"),
        unread_only,
        cx.listener(|this, _e: &ClickEvent, _w, cx| {
            this.notifications_unread_only = !this.notifications_unread_only;
            this.notifications_loading = true;
            this.backend.send(MessageToBackend::RequestNotifications {
                offset: 0,
                unread_only: this.notifications_unread_only,
            });
            cx.notify();
        }),
    )
}

fn empty(ui: &LauncherUI) -> AnyElement {
    div()
        .py(px(28.))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.))
        .child(ic("inbox", 20., TEXT_MUTED))
        .child(div().text_size(px(12.)).text_color(rgb(TEXT_MUTED)).child(
            if ui.notifications_loading {
                t("launcher-loading")
            } else {
                t("launcher-notifications-empty")
            },
        ))
        .into_any_element()
}

fn card(n: &Notification, cx: &mut Cx) -> AnyElement {
    let id = n.id;
    let unread = !n.read;
    let (icon, colour) = look(n.level);
    let title = translated(n.title_key.as_deref(), &n.title, &n.args);
    let body = translated(n.body_key.as_deref(), &n.body, &n.args);
    let link = n.link.clone();

    div()
        .id(gpui::SharedString::from(format!("notif-{id}")))
        .p(px(12.))
        .rounded(px(R_SM))
        .flex()
        .gap(px(12.))
        .cursor_pointer()
        .bg(if unread {
            rgba((ACCENT << 8) | 0x10)
        } else {
            rgba(0xffffff05)
        })
        .hover(|d| d.bg(rgba(0xffffff12)))
        .child(div().pt(px(2.)).child(ic(icon, 16., colour)))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(if unread {
                            FontWeight::BOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(rgb(TEXT_PRIMARY))
                        .child(title),
                )
                .when(!body.is_empty(), |d| {
                    d.child(
                        div()
                            .text_size(px(11.))
                            .text_color(rgb(TEXT_SECONDARY))
                            .child(body),
                    )
                })
                .when(n.repeat_count > 1, |d| {
                    d.child(
                        div()
                            .text_size(px(10.))
                            .text_color(rgb(TEXT_MUTED))
                            .child(format!("×{}", n.repeat_count)),
                    )
                }),
        )
        // Clicking marks it read and, when the card leads somewhere, opens that
        // in a browser: the destination is a page on the site, and the launcher
        // has no second copy of it.
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            if unread {
                this.backend
                    .send(MessageToBackend::MarkNotificationRead { id });
                if let Some(n) = this.notifications.iter_mut().find(|n| n.id == id) {
                    n.read = true;
                }
            }
            if let Some(href) = &link {
                // Путь в карточке — это страница сайта, а адреса сайта у
                // лаунчера нет: впечатан только мастер. Он и переводит на
                // сайт, как при входе через браузер. Раньше путь клеился к
                // адресу мастера, и ссылка вела на API вместо страницы.
                let url = if href.starts_with("http") {
                    href.clone()
                } else {
                    format!(
                        "{}/go?to={}",
                        this.config.master_url.trim_end_matches('/'),
                        urlencoding::encode(href)
                    )
                };
                let _ = open::that_detached(url);
            }
            cx.notify();
        }))
        .into_any_element()
}

fn look(level: Level) -> (&'static str, u32) {
    match level {
        Level::Urgent => ("circle-alert", ERROR),
        Level::Important => ("triangle-alert", WARNING),
        Level::Normal => ("info", BLUE),
    }
}

/// Prefer our own catalogue over the master's finished string.
///
/// The master translates with the locale it runs in. For the site that is
/// right — it reads the same catalogue from the master — but the launcher has
/// its own, in the language the player chose here, and showing them Russian
/// because the server runs in Russian is not a translation.
fn translated(
    key: Option<&str>,
    fallback: &str,
    args: &std::collections::BTreeMap<String, String>,
) -> String {
    let Some(key) = key else {
        return fallback.to_string();
    };
    let translated = if args.is_empty() {
        t(key)
    } else {
        let mut fluent = i18n::FluentArgs::new();
        for (k, v) in args {
            fluent.set(k.clone(), v.clone());
        }
        i18n::t_args(key, &fluent)
    };
    // `t` hands back the key itself when it knows nothing about it. The
    // master's own rendering is a better answer than a raw key.
    if translated == key {
        fallback.to_string()
    } else {
        translated
    }
}
