//! The punishments tab: what was handed out, by whom, and whether it still holds.

use super::account::{empty, loading, row, scroll};
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight};
use i18n::t;

pub(super) fn punishments(ui: &LauncherUI) -> AnyElement {
    if ui.punishments.is_empty() {
        return if ui.account_loaded.contains("punishments") {
            empty("circle-check", t("account-punishments-none"))
        } else {
            loading()
        };
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
                                .child(punishment_kind(&p.kind)),
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

/// The master sends its own identifiers (`ban`, `server_ban`); the catalog
/// has a word for each. One it doesn't know yet still shows, as it came.
fn punishment_kind(kind: &str) -> String {
    let key = format!("punishment-kind-{}", kind.replace('_', "-"));
    if i18n::has_key(&key) {
        t(&key)
    } else {
        kind.to_uppercase()
    }
}
