// Over 150 lines: a list of conversations and the open one side by side. They
// are one screen, and the list is what closes the open thread.
//! Direct messages.
//!
//! Two panes rather than a list that replaces itself: a conversation in the
//! launcher is usually with staff about something happening right now, and
//! losing the list to read one message means clicking back to see who else
//! wrote.

use super::common::{panel, Cx};
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::{DmMessageView, DmThreadOpen, DmThreadView, MessageToBackend};
use gpui::{
    div, img, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, ObjectFit,
    SharedString,
};
use i18n::t;

pub fn page(ui: &mut LauncherUI, cx: &mut Cx) -> AnyElement {
    // Once on entry, not "while the list is empty": this runs every frame, and
    // an account with no conversations would ask for them sixty times a second.
    if !ui.dm_requested {
        ui.dm_requested = true;
        ui.backend.send(MessageToBackend::RequestDmThreads);
    }
    // Heads are ordinary images from the master; the cache is keyed by URL.
    let faces: Vec<String> = ui
        .dm_threads
        .iter()
        .filter_map(|t| t.avatar_url.clone())
        .chain(ui.dm_open.iter().filter_map(|o| o.avatar_url.clone()))
        .collect();
    for url in faces {
        ui.ensure_optional_mod_icon_loaded(Some(url), cx);
    }

    div()
        .size_full()
        .bg(rgb(CONTENT_FALLBACK))
        .p(px(32.))
        .flex()
        .gap(px(16.))
        .child(threads(ui, cx))
        .child(thread(ui, cx))
        .into_any_element()
}

/// Somebody's head, or their initial while there is no picture.
pub(super) fn face(ui: &LauncherUI, url: &Option<String>, name: &str, size: f32) -> AnyElement {
    let frame = div()
        .size(px(size))
        .rounded(px(R_SM))
        .overflow_hidden()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center();

    if let Some(image) = url
        .as_ref()
        .and_then(|u| ui.optional_mod_icons.get(u).cloned())
    {
        return frame
            .child(img(image).size_full().object_fit(ObjectFit::Cover))
            .into_any_element();
    }
    frame
        .bg(rgba(0xffffff12))
        .border_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(size * 0.4))
                .text_color(rgb(TEXT_SECONDARY))
                .child(super::common::initial(name)),
        )
        .into_any_element()
}

fn threads(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let open = ui.dm_open.as_ref().map(|o| o.peer);
    let rows: Vec<AnyElement> = ui
        .dm_threads
        .iter()
        .map(|t| thread_row(ui, t, open == Some(t.peer_id), cx))
        .collect();

    panel()
        .id("dm-threads")
        .w(px(300.))
        .flex_shrink_0()
        .overflow_y_scroll()
        .p(px(8.))
        .flex()
        .flex_col()
        .gap(px(2.))
        .children(rows)
        .when(ui.dm_threads.is_empty(), |d| {
            d.child(
                div()
                    .py(px(40.))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(10.))
                    .child(ic(
                        if ui.dm_loaded { "inbox" } else { "hourglass" },
                        24.,
                        TEXT_MUTED,
                    ))
                    .child(div().text_size(px(11.)).text_color(rgb(TEXT_MUTED)).child(
                        if ui.dm_loaded {
                            t("messages-none")
                        } else {
                            t("launcher-loading")
                        },
                    )),
            )
        })
        .into_any_element()
}

fn thread_row(ui: &LauncherUI, row: &DmThreadView, active: bool, cx: &mut Cx) -> AnyElement {
    let peer = row.peer_id;
    let unread = row.unread;

    div()
        .id(SharedString::from(format!("dm-thread-{peer}")))
        .p(px(8.))
        .rounded(px(R_SM))
        .cursor_pointer()
        .bg(if active {
            rgba((CTA << 8) | 0x14)
        } else {
            rgba(0x00000000)
        })
        .hover(|d| d.bg(rgba(0xffffff0c)))
        .flex()
        .items_center()
        .gap(px(10.))
        .child(face(ui, &row.avatar_url, &row.peer_name, 36.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_size(px(12.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(TEXT_PRIMARY))
                                .truncate()
                                .child(row.peer_name.clone()),
                        )
                        .child(
                            div()
                                .flex_shrink_0()
                                .text_size(px(9.))
                                .text_color(rgb(TEXT_MUTED))
                                .child(super::common::short_time(row.last_message_at)),
                        ),
                )
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(rgb(if unread > 0 {
                            TEXT_SECONDARY
                        } else {
                            TEXT_MUTED
                        }))
                        .truncate()
                        .child(row.preview.clone()),
                ),
        )
        .when(unread > 0, |d| d.child(badge(unread)))
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.backend
                .send(MessageToBackend::RequestDmThread { peer });
            cx.notify();
        }))
        .into_any_element()
}

fn badge(unread: i64) -> AnyElement {
    div()
        .flex_shrink_0()
        .min_w(px(18.))
        .h(px(18.))
        .px(px(5.))
        .rounded(px(9.))
        .bg(rgb(ACCENT))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_size(px(9.))
                .text_color(rgb(BG_WINDOW))
                .child(unread.to_string()),
        )
        .into_any_element()
}

fn thread(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    // By reference: cloning the conversation copied every message on every frame.
    let Some(open) = ui.dm_open.as_ref() else {
        return panel()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(12.))
            .child(ic("inbox", 26., TEXT_MUTED))
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(rgb(TEXT_MUTED))
                    .child(t("messages-pick")),
            )
            .into_any_element();
    };

    let peer = open.peer;
    let empty = open.messages.is_empty();
    let mut rows: Vec<AnyElement> = Vec::new();
    let mut last_day = String::new();
    for m in &open.messages {
        // A separator only where the day changes: a chat that stamps a date on
        // every line reads like a log, not a conversation.
        let day = super::common::short_day(m.at);
        if day != last_day {
            rows.push(day_divider(&day));
            last_day = day;
        }
        rows.push(bubble(ui, open, m));
    }

    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(head(ui, open))
        .child(
            panel()
                .id("dm-messages")
                .track_scroll(&ui.dm_scroll)
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .p(px(16.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .children(rows)
                .when(empty, |d| {
                    d.justify_center().items_center().child(
                        div()
                            .text_size(px(12.))
                            .text_color(rgb(TEXT_MUTED))
                            .child(t("messages-empty")),
                    )
                }),
        )
        .child(super::compose::box_for(
            ui,
            "dm-compose",
            move |this, body| {
                this.backend.send(MessageToBackend::SendDm { peer, body });
            },
            cx,
        ))
        .into_any_element()
}

fn head(ui: &LauncherUI, open: &DmThreadOpen) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(face(ui, &open.avatar_url, &open.peer_name, 32.))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(14.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .child(open.peer_name.clone()),
        )
        .into_any_element()
}

fn day_divider(day: &str) -> AnyElement {
    div()
        .py(px(4.))
        .flex()
        .items_center()
        .gap(px(8.))
        .child(div().flex_1().h(px(1.)).bg(rgb(BORDER)))
        .child(
            div()
                .flex_shrink_0()
                .text_size(px(9.))
                .text_color(rgb(TEXT_MUTED))
                .child(day.to_string()),
        )
        .child(div().flex_1().h(px(1.)).bg(rgb(BORDER)))
        .into_any_element()
}

fn bubble(ui: &LauncherUI, open: &DmThreadOpen, m: &DmMessageView) -> AnyElement {
    let mine = m.mine;

    div()
        .flex()
        .gap(px(8.))
        .when(mine, |d| d.justify_end())
        // Their head beside their words. Ours is not drawn: on our own messages
        // it is one more copy of a face we already know.
        .when(!mine, |d| {
            d.child(face(ui, &open.avatar_url, &m.author_name, 28.))
        })
        .child(
            div()
                .max_w(px(460.))
                .px(px(12.))
                .py(px(8.))
                .rounded(px(R_MD))
                .bg(if mine {
                    rgba((CTA << 8) | 0x1a)
                } else {
                    rgba(0xffffff0c)
                })
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(TEXT_PRIMARY))
                        .child(m.body.clone()),
                )
                .child(
                    div()
                        .text_size(px(9.))
                        .text_color(rgb(TEXT_MUTED))
                        .when(mine, |d| d.text_right())
                        .child(super::common::short_time(m.at)),
                ),
        )
        .into_any_element()
}

/// Unread across every conversation — the dot on the sidebar button.
pub fn unread_total(ui: &LauncherUI) -> i64 {
    ui.dm_threads.iter().map(|t| t.unread).sum()
}
