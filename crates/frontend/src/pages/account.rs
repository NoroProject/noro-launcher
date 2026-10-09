//! Punishments, support tickets and the rule book.
//!
//! In the launcher and not only on the site because this is when they matter.
//! A ban is read at the moment the game refuses to start, a staff reply is
//! waited for while the launcher is open, and a rule is looked up right after
//! the punishment that cited it — all of it with the window already in front of
//! the person.

use super::account_punishments::punishments;
use super::account_tickets::tickets;
use super::common::{panel, Cx};
use crate::icons::ic;
use crate::state::{AccountTab, LauncherUI};
use crate::theme::*;
use bridge::MessageToBackend;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, SharedString};
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

pub(super) fn loading() -> AnyElement {
    empty("hourglass", t("launcher-loading"))
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
