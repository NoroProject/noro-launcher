// Over 150 lines: the rule book is a search box, its sections and one rule
// card, and the card is what the other two exist to find.
//! The project's rule book.
//!
//! Three things the previous version did not have, and each of them is why
//! somebody opens this screen. A search box, because a rule is looked up after
//! something happened, not read front to back. Sections, because thirty rules
//! in one column are a wall. And the punishments each rule allows — for most
//! people that is the actual question, and a rule book without them reads as a
//! list of wishes.

use super::common::Cx;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::{RuleView, SanctionView};
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use i18n::t;

pub fn view(ui: &mut LauncherUI, cx: &mut Cx) -> AnyElement {
    let needle = ui.rules_query.trim().to_lowercase();
    let matching: Vec<&RuleView> = ui
        .rules
        .iter()
        .filter(|r| {
            needle.is_empty()
                || r.title.to_lowercase().contains(&needle)
                || r.code.to_lowercase().contains(&needle)
                || r.description.to_lowercase().contains(&needle)
        })
        .collect();

    // Sections in the order the master sent them; a rule with no section goes
    // last under an empty heading rather than vanishing.
    let mut sections: Vec<(String, Vec<&RuleView>)> = Vec::new();
    for rule in matching {
        match sections.iter_mut().find(|(name, _)| name == &rule.category) {
            Some((_, rules)) => rules.push(rule),
            None => sections.push((rule.category.clone(), vec![rule])),
        }
    }

    let empty = sections.is_empty();
    let blocks: Vec<AnyElement> = sections
        .into_iter()
        .map(|(name, rules)| section(&name, &rules))
        .collect();

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(search(ui, cx))
        .child(if empty && !ui.account_loaded.contains("rules") {
            super::account::loading()
        } else if empty {
            super::account::empty("info", t("account-rules-none"))
        } else {
            super::account::scroll("account-rules-scroll", blocks)
        })
        .into_any_element()
}

fn search(ui: &mut LauncherUI, cx: &mut Cx) -> AnyElement {
    let query = ui.rules_query.clone();
    let focus = ui
        .rules_focus
        .get_or_insert_with(|| cx.focus_handle())
        .clone();
    let focus_for_click = focus.clone();

    div()
        .id("rules-search")
        .track_focus(&focus)
        .h(px(36.))
        .px(px(12.))
        .rounded(px(R_SM))
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(BORDER))
        .focus(|s| s.border_color(rgb(CTA)))
        .flex()
        .items_center()
        .gap(px(8.))
        .cursor_text()
        .on_click(cx.listener(move |_this, _e: &ClickEvent, window, cx| {
            focus_for_click.focus(window, cx);
            cx.notify();
        }))
        .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _w, cx| {
            if let Some(text) = super::common::pasted(event, cx) {
                this.rules_query.push_str(&text);
                cx.notify();
                return;
            }
            match event.keystroke.key.as_str() {
                "backspace" => {
                    this.rules_query.pop();
                }
                "space" => this.rules_query.push(' '),
                "escape" => this.rules_query.clear(),
                _ => {
                    if let Some(ch) = event.keystroke.key_char.as_deref() {
                        this.rules_query.push_str(ch);
                    }
                }
            }
            cx.notify();
        }))
        .child(ic("search", 14., TEXT_MUTED))
        .child(
            div()
                .flex_1()
                .text_size(px(12.))
                .text_color(rgb(if query.is_empty() {
                    TEXT_MUTED
                } else {
                    TEXT_PRIMARY
                }))
                .child(if query.is_empty() {
                    t("account-rules-search")
                } else {
                    format!("{query}_")
                }),
        )
        .into_any_element()
}

fn section(name: &str, rules: &[&RuleView]) -> AnyElement {
    let cards: Vec<AnyElement> = rules.iter().map(|r| card(r)).collect();

    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .when(!name.is_empty(), |d| {
            d.child(
                div()
                    .pt(px(4.))
                    .font_family(FONT_PIXEL_ALT)
                    .text_size(px(11.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(TEXT_MUTED))
                    .child(name.to_uppercase()),
            )
        })
        .children(cards)
        .into_any_element()
}

fn card(rule: &RuleView) -> AnyElement {
    let chips: Vec<AnyElement> = rule.sanctions.iter().map(sanction).collect();

    super::account::row(
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .px(px(6.))
                            .rounded(px(R_SM))
                            .bg(rgba(0x00000040))
                            .text_size(px(10.))
                            .text_color(rgb(ACCENT))
                            .child(rule.code.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .font_family(FONT_PIXEL_ALT)
                            .text_size(px(13.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(TEXT_PRIMARY))
                            .child(rule.title.clone()),
                    ),
            )
            .when(!rule.description.is_empty(), |d| {
                d.child(
                    div()
                        .text_size(px(11.))
                        .text_color(rgb(TEXT_SECONDARY))
                        .child(rule.description.clone()),
                )
            })
            .when(!chips.is_empty(), |d| {
                d.child(div().flex().flex_wrap().gap(px(4.)).children(chips))
            })
            .into_any_element(),
    )
}

/// One punishment, with how long it runs.
fn sanction(s: &SanctionView) -> AnyElement {
    let colour = match s.kind.as_str() {
        "warn" => WARNING,
        "mute" => BLUE,
        _ => ERROR,
    };
    let label = if s.label.is_empty() {
        s.kind.to_uppercase()
    } else {
        s.label.clone()
    };

    div()
        .id(SharedString::from(format!("sanction-{}-{label}", s.kind)))
        .px(px(8.))
        .h(px(20.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .gap(px(6.))
        .bg(rgba(0xffffff08))
        .border_1()
        .border_color(rgb(colour))
        .child(
            div()
                .text_size(px(10.))
                .text_color(rgb(colour))
                .child(label),
        )
        .when_some(duration(s), |d, text| {
            d.child(
                div()
                    .text_size(px(10.))
                    .text_color(rgb(TEXT_MUTED))
                    .child(text),
            )
        })
        .into_any_element()
}

/// The range in words. No upper bound reads as permanent, which is the part
/// people want to know.
fn duration(s: &SanctionView) -> Option<String> {
    match (s.min_minutes, s.max_minutes) {
        (None, None) => None,
        (_, None) => Some(t("account-forever")),
        (None, Some(max)) => Some(humanize(max)),
        (Some(min), Some(max)) if min == max => Some(humanize(max)),
        (Some(min), Some(max)) => Some(format!("{} – {}", humanize(min), humanize(max))),
    }
}

fn humanize(minutes: i64) -> String {
    let mut args = i18n::FluentArgs::new();
    if minutes >= 1440 {
        args.set("n", (minutes / 1440).to_string());
        i18n::t_args("duration-days", &args)
    } else if minutes >= 60 {
        args.set("n", (minutes / 60).to_string());
        i18n::t_args("duration-hours", &args)
    } else {
        args.set("n", minutes.to_string());
        i18n::t_args("duration-minutes", &args)
    }
}
