// Over 150 lines: the bell and the panel it opens, with the panel's header,
// filter and paging. Both read the open flag and the unread count.
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
use super::notification_card::card;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::MessageToBackend;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight};
use i18n::t;

/// The bell, with an unread count when there is one.
pub fn bell(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let unread = ui.unread;
    let open = ui.notifications_open;

    div()
        .id("notifications-bell")
        .tooltip(crate::components::hint(t("hint-notifications")))
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
    let more = (ui.notifications.len() as i64) < ui.notifications_total;

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
                    // Height by content: a fixed one stretched the panel from the bell to
                    // the top of the window for a single "empty" line.
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
                            .when(more, |d| d.child(load_more(ui, cx)))
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
                .tooltip(crate::components::hint(t("hint-read-all")))
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

/// The next page of the feed. Only the first page was ever requested, so
/// anything older than it couldn't be reached at all.
fn load_more(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let loading = ui.notifications_loading;
    div()
        .id("notifications-more")
        .h(px(32.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(R_SM))
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(11.))
        .text_color(rgb(TEXT_SECONDARY))
        .when(!loading, |d| {
            d.cursor_pointer()
                .hover(|d| d.bg(rgba(0xffffff10)).text_color(rgb(TEXT_PRIMARY)))
        })
        .child(if loading {
            t("launcher-loading")
        } else {
            t("launcher-notifications-more")
        })
        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
            if this.notifications_loading {
                return;
            }
            this.notifications_loading = true;
            this.backend.send(MessageToBackend::RequestNotifications {
                offset: this.notifications.len() as u32,
                unread_only: this.notifications_unread_only,
            });
            cx.notify();
        }))
        .into_any_element()
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
