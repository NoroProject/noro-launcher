//! "My suggestions" under the optional mods: what the player suggested for this
//! server and what became of it.
//!
//! Before this, a suggestion left the launcher and was never heard of again —
//! only the build's admin saw its status. Who decided is not shown: the master
//! does not tell the author, and the decision belongs to the build.

use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, FontWeight};
use i18n::t;
use schema::{MyModSuggestion, SuggestionStatus};
use uuid::Uuid;

/// The section, or nothing when the player has suggested nothing here.
pub(super) fn section(ui: &LauncherUI, server_id: Uuid) -> Option<AnyElement> {
    let items = ui.my_suggestions.get(&server_id)?;
    if items.is_empty() {
        return None;
    }
    let rows: Vec<AnyElement> = items.iter().map(row).collect();
    Some(
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(ic("send", 14., ACCENT))
                    .child(
                        div()
                            .font_family(FONT_PIXEL_ALT)
                            .text_size(px(14.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(CTA))
                            .child(t("mods-my-suggestions")),
                    ),
            )
            .child(
                div()
                    .rounded(px(R_MD))
                    .bg(rgb(BG_PANEL))
                    .border_1()
                    .border_color(rgb(BORDER))
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .children(rows),
            )
            .into_any_element(),
    )
}

fn row(s: &MyModSuggestion) -> AnyElement {
    let reason = s
        .decision_reason
        .clone()
        .filter(|r| s.status == SuggestionStatus::Rejected && !r.trim().is_empty());
    div()
        .px(px(16.))
        .py(px(12.))
        .border_b_1()
        .border_color(rgb(BORDER))
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
                        .flex_1()
                        .min_w_0()
                        .text_size(px(13.))
                        .text_color(rgb(TEXT_PRIMARY))
                        .truncate()
                        .child(s.title.clone()),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .text_size(px(10.))
                        .text_color(rgb(TEXT_MUTED))
                        .child(super::common::short_date(s.created_at.timestamp())),
                )
                .child(status_chip(s.status)),
        )
        .children(reason.map(|reason| {
            let mut args = i18n::FluentArgs::new();
            args.set("reason", reason);
            div()
                .text_size(px(12.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(i18n::t_args("suggestion-reason", &args))
        }))
        .into_any_element()
}

fn status_chip(status: SuggestionStatus) -> AnyElement {
    let (key, colour) = match status {
        SuggestionStatus::Pending => ("suggestion-status-pending", WARNING),
        SuggestionStatus::Approved => ("suggestion-status-approved", BLUE),
        SuggestionStatus::Installed => ("suggestion-status-installed", SUCCESS),
        SuggestionStatus::Rejected => ("suggestion-status-rejected", ERROR),
        SuggestionStatus::Unknown => ("suggestion-status-unknown", TEXT_MUTED),
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
