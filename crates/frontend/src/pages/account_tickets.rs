// Over 150 lines: the list, one row and the status chip in it; the row has no
// other use.
//! The tickets tab: the player's support tickets and the button that opens one.

use super::account::{empty, loading, row, scroll};
use super::common::Cx;
use crate::components::btn;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::MessageToBackend;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use i18n::t;

pub(super) fn tickets(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    if let Some((id, subject, status, messages)) = ui.ticket_open.clone() {
        return super::account_thread::ticket(ui, id, subject, status, messages, cx);
    }

    let rows: Vec<AnyElement> = ui.tickets.iter().map(|t| ticket_row(t, cx)).collect();

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(new_ticket_button(cx))
        .child(
            if rows.is_empty() && !ui.account_loaded.contains("tickets") {
                loading()
            } else if rows.is_empty() {
                empty("inbox", t("account-tickets-none"))
            } else {
                scroll("account-tickets-scroll", rows)
            },
        )
        .into_any_element()
}

fn ticket_row(ticket: &bridge::TicketView, cx: &mut Cx) -> AnyElement {
    let id = ticket.id;
    let unread = ticket.unread;

    div()
        .id(SharedString::from(format!("ticket-{id}")))
        .cursor_pointer()
        .child(row(div()
            .flex()
            .items_center()
            .gap(px(10.))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            // The number first: support asks "which ticket?"
                            // and a subject is not an answer to that.
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .px(px(6.))
                                    .rounded(px(R_SM))
                                    .bg(rgba(0x00000040))
                                    .text_size(px(10.))
                                    .text_color(rgb(ACCENT))
                                    .child(format!("#{}", ticket.number)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .text_size(px(13.))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(TEXT_PRIMARY))
                                    .truncate()
                                    .child(ticket.subject.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .child(ticket_status(&ticket.status))
                            .child(
                                div()
                                    .text_size(px(10.))
                                    .text_color(rgb(TEXT_MUTED))
                                    .child(super::common::short_date(ticket.last_message_at)),
                            ),
                    ),
            )
            .when(unread > 0, |d| {
                d.child(
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
                        ),
                )
            })
            .into_any_element()))
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.backend.send(MessageToBackend::RequestTicket { id });
            cx.notify();
        }))
        .into_any_element()
}

/// `open`, `answered`, `closed` in the player's language, for the list and
/// the open ticket alike.
pub(super) fn ticket_status_label(status: &str) -> String {
    let key = format!("tickets-status-{status}");
    if i18n::has_key(&key) {
        t(&key)
    } else {
        status.to_string()
    }
}

/// Coloured so the one state that wants something from the player stands out.
fn ticket_status(status: &str) -> AnyElement {
    let colour = match status {
        "answered" => SUCCESS,
        "closed" => TEXT_MUTED,
        _ => WARNING,
    };
    div()
        .px(px(6.))
        .h(px(18.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .bg(rgba(0x00000040))
        .child(
            div()
                .text_size(px(10.))
                .text_color(rgb(colour))
                .child(ticket_status_label(status)),
        )
        .into_any_element()
}

/// Opening a ticket needs a subject and a body, and the launcher has no text
/// dialog. So the first message is composed in the thread view, which already
/// has a field: the button creates the draft and the reply box sends it.
fn new_ticket_button(cx: &mut Cx) -> AnyElement {
    btn(
        "ticket-new",
        t("account-ticket-new"),
        true,
        cx.listener(|this, _e: &ClickEvent, _w, cx| {
            this.ticket_open = Some((
                uuid::Uuid::nil(),
                t("account-ticket-new"),
                "draft".to_string(),
                Vec::new(),
            ));
            this.compose.clear();
            cx.notify();
        }),
    )
    .into_any_element()
}
