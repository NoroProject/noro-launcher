// Over 150 lines: three lists that answer one question — where this player
// stands with the project — plus the tabs that switch between them.
//! Punishments, support tickets and the rule book.
//!
//! In the launcher and not only on the site because this is when they matter.
//! A ban is read at the moment the game refuses to start, a staff reply is
//! waited for while the launcher is open, and a rule is looked up right after
//! the punishment that cited it — all of it with the window already in front of
//! the person.

use super::common::{panel, Cx};
use crate::components::btn;
use crate::icons::ic;
use crate::state::{AccountTab, LauncherUI};
use crate::theme::*;
use bridge::MessageToBackend;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use i18n::t;

pub fn page(ui: &mut LauncherUI, cx: &mut Cx) -> AnyElement {
    load_once(ui);

    div()
        .size_full()
        .bg(rgb(CONTENT_FALLBACK))
        .p(px(32.))
        .flex()
        .flex_col()
        .gap(px(16.))
        .child(header(ui, cx))
        .child(match ui.account_tab {
            AccountTab::Punishments => punishments(ui),
            AccountTab::Tickets => tickets(ui, cx),
            AccountTab::Rules => super::account_rules::view(ui, cx),
        })
        .into_any_element()
}

/// Each list is asked for once, when its tab is first opened. Fetching all
/// three on entry would be three requests for a screen where two are usually
/// never looked at.
fn load_once(ui: &mut LauncherUI) {
    let (key, message) = match ui.account_tab {
        AccountTab::Punishments => ("punishments", MessageToBackend::RequestPunishments),
        AccountTab::Tickets => ("tickets", MessageToBackend::RequestTickets),
        AccountTab::Rules => ("rules", MessageToBackend::RequestRules),
    };
    // By the flag, not by an empty list: an empty answer is an answer too, and
    // the contents can't tell it apart from "not asked yet".
    if ui.account_requested.insert(key) {
        ui.backend.send(message);
    }
}

fn header(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let tabs = [
        (AccountTab::Punishments, "account-punishments"),
        (AccountTab::Tickets, "account-tickets"),
        (AccountTab::Rules, "account-rules"),
    ];

    let chips: Vec<AnyElement> = tabs
        .into_iter()
        .map(|(tab, key)| {
            crate::components::segment(
                SharedString::from(format!("account-tab-{key}")),
                None,
                t(key),
                ui.account_tab == tab,
                cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    this.account_tab = tab;
                    this.ticket_open = None;
                    cx.notify();
                }),
            )
        })
        .collect();

    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .children(chips)
        .into_any_element()
}

fn punishments(ui: &LauncherUI) -> AnyElement {
    if ui.punishments.is_empty() {
        return empty("circle-check", t("account-punishments-none"));
    }

    let rows: Vec<AnyElement> = ui.punishments.iter().map(punishment).collect();
    scroll("account-punishments-scroll", rows)
}

/// One punishment: what it is, for what, by whom and until when.
///
/// A spent punishment stays in the list but goes grey and says so. Dropping it
/// would leave a record the player cannot check against what staff can see.
fn punishment(p: &bridge::PunishmentView) -> AnyElement {
    let (icon, colour) = match p.kind.as_str() {
        "warn" => ("triangle-alert", WARNING),
        "mute" => ("volume-x", BLUE),
        _ => ("ban", ERROR),
    };
    let colour = if p.active { colour } else { TEXT_MUTED };
    let until = match p.expires_at {
        Some(at) => super::common::short_date(at),
        None => t("account-forever"),
    };

    row(div()
        .flex()
        .gap(px(12.))
        .child(div().pt(px(2.)).child(ic(icon, 18., colour)))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(5.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .font_family(FONT_PIXEL_ALT)
                                .text_size(px(13.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(colour))
                                .child(p.kind.to_uppercase()),
                        )
                        .children(p.rule_code.clone().map(|code| {
                            div()
                                .px(px(6.))
                                .rounded(px(R_SM))
                                .bg(rgba(0x00000040))
                                .text_size(px(10.))
                                .text_color(rgb(ACCENT))
                                .child(code)
                        }))
                        .child(div().flex_1())
                        .child(status_chip(p.active)),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(rgb(if p.active {
                            TEXT_PRIMARY
                        } else {
                            TEXT_SECONDARY
                        }))
                        .child(p.reason.clone()),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(10.))
                        .text_color(rgb(TEXT_MUTED))
                        .child(ic("user", 11., TEXT_MUTED))
                        .child(p.actor.clone())
                        .child(div().child("·"))
                        .child(super::common::short_date(p.created_at))
                        .child(ic("arrow-right", 11., TEXT_MUTED))
                        .child(until),
                ),
        )
        .into_any_element())
}

fn status_chip(active: bool) -> AnyElement {
    let (key, colour) = if active {
        ("account-punishment-active", ERROR)
    } else {
        ("account-punishment-over", SUCCESS)
    };
    div()
        .flex_shrink_0()
        .px(px(8.))
        .h(px(20.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .bg(rgba(0x00000040))
        .child(
            div()
                .text_size(px(10.))
                .text_color(rgb(colour))
                .child(t(key)),
        )
        .into_any_element()
}

fn tickets(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
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
        .child(if rows.is_empty() {
            empty("inbox", t("account-tickets-none"))
        } else {
            scroll("account-tickets-scroll", rows)
        })
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

/// The master's own words — `open`, `answered`, `closed` — coloured so the one
/// state that wants something from the player stands out.
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
                .child(status.to_string()),
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

pub(super) fn row(content: AnyElement) -> AnyElement {
    div()
        .p(px(12.))
        .rounded(px(R_SM))
        .bg(rgba(0xffffff08))
        .border_1()
        .border_color(rgb(BORDER))
        .child(content)
        .into_any_element()
}

pub(super) fn scroll(id: &'static str, rows: Vec<AnyElement>) -> AnyElement {
    panel()
        .id(id)
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .p(px(12.))
        .flex()
        .flex_col()
        .gap(px(8.))
        .children(rows)
        .into_any_element()
}

pub(super) fn empty(icon: &'static str, text: String) -> AnyElement {
    panel()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(12.))
        .child(ic(icon, 28., TEXT_MUTED))
        .child(
            div()
                .text_size(px(12.))
                .text_color(rgb(TEXT_MUTED))
                .child(text),
        )
        .into_any_element()
}
