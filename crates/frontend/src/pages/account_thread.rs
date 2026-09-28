//! An open support ticket: the thread and the reply box.
//!
//! A ticket with no id yet is a draft — the button on the list makes one, and
//! the first thing sent opens the ticket for real. The launcher has no text
//! dialog, so composing a subject and a body in two fields before anything
//! exists would mean building one; a draft thread reuses the box that is
//! already here.

use super::common::{panel, Cx};
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::{MessageToBackend, TicketMessageView};
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use i18n::t;
use uuid::Uuid;

pub fn ticket(
    ui: &LauncherUI,
    id: Uuid,
    subject: String,
    status: String,
    messages: Vec<TicketMessageView>,
    cx: &mut Cx,
) -> AnyElement {
    let bubbles: Vec<AnyElement> = messages.iter().map(bubble).collect();
    let draft = id.is_nil();

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(head(subject, status, cx))
        .child(
            panel()
                .id("ticket-thread")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .p(px(12.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .children(bubbles)
                .when(messages.is_empty(), |d| {
                    d.child(
                        div()
                            .py(px(32.))
                            .flex()
                            .justify_center()
                            .text_size(px(12.))
                            .text_color(rgb(TEXT_MUTED))
                            .child(t(if draft {
                                "account-ticket-draft-hint"
                            } else {
                                "account-tickets-none"
                            })),
                    )
                }),
        )
        .child(super::compose::box_for(
            ui,
            "ticket-compose",
            move |this, text| {
                if draft {
                    // The first line is both the subject and the body: a support
                    // request that starts "the game will not start after the
                    // update" needs no separate title.
                    let subject: String = text.chars().take(80).collect();
                    this.backend.send(MessageToBackend::OpenTicket {
                        subject,
                        content: text,
                    });
                    this.ticket_open = None;
                } else {
                    this.backend
                        .send(MessageToBackend::ReplyTicket { id, content: text });
                }
            },
            cx,
        ))
        .into_any_element()
}

fn head(subject: String, status: String, cx: &mut Cx) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(
            div()
                .id("ticket-back")
                .size(px(30.))
                .rounded(px(R_SM))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|d| d.bg(rgba(0xffffff12)))
                .child(ic("arrow-left", 15., TEXT_MUTED))
                .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    this.ticket_open = None;
                    cx.notify();
                })),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(14.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(CTA))
                .truncate()
                .child(subject),
        )
        .child(
            div()
                .px(px(8.))
                .py(px(2.))
                .rounded(px(R_SM))
                .bg(rgba(0x00000040))
                .text_size(px(10.))
                .text_color(rgb(TEXT_MUTED))
                .child(status),
        )
        .into_any_element()
}

fn bubble(m: &TicketMessageView) -> AnyElement {
    div()
        .id(SharedString::from(format!("ticket-msg-{}", m.at)))
        .p(px(10.))
        .rounded(px(R_SM))
        .bg(if m.staff {
            rgba((ACCENT << 8) | 0x14)
        } else {
            rgba(0xffffff08)
        })
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .text_size(px(11.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(if m.staff { ACCENT } else { TEXT_SECONDARY }))
                        .child(m.author.clone()),
                )
                .child(
                    div()
                        .text_size(px(10.))
                        .text_color(rgb(TEXT_MUTED))
                        .child(super::common::short_date(m.at)),
                ),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(rgb(TEXT_PRIMARY))
                .child(m.content.clone()),
        )
        .into_any_element()
}
