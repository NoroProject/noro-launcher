// Over 150 lines: one row of the list carries its state, its explanation and
// its two actions, and the explanation is the reason the screen exists.
//! What this player added to the build, and why some of it is not running.
//!
//! The list shows everything installed, including what is not active. A mod the
//! build has since taken over, one blocked by staff and one that does not fit
//! the current Minecraft version all disappear from `mods/` — and a row that
//! vanishes with no explanation reads as a launcher fault, which is how people
//! end up reinstalling the same mod three times.

use super::common::Cx;
use super::content_card::mod_avatar;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::MessageToBackend;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use i18n::t;
use schema::personal::{PersonalItem, PersonalState};
use uuid::Uuid;

pub fn view(ui: &mut LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let items = ui
        .personal_content
        .get(&server_id)
        .cloned()
        .unwrap_or_default();
    for url in items.iter().filter_map(|i| i.content.icon_url.clone()) {
        ui.ensure_optional_mod_icon_loaded(Some(url), cx);
    }

    let rows: Vec<AnyElement> = items
        .iter()
        .map(|item| row(ui, server_id, item, cx))
        .collect();

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(
            div()
                .id("installed-scroll")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .rounded(px(R_MD))
                .bg(rgb(BG_PANEL))
                .border_1()
                .border_color(rgb(BORDER))
                .p(px(12.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .children(rows)
                .when(items.is_empty(), |d| d.child(empty())),
        )
        .into_any_element()
}

fn empty() -> AnyElement {
    div()
        .py(px(56.))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(12.))
        .child(ic("box", 26., TEXT_MUTED))
        .child(
            div()
                .text_size(px(12.))
                .text_color(rgb(TEXT_MUTED))
                .child(t("content-installed-empty")),
        )
        .into_any_element()
}

fn row(ui: &LauncherUI, server_id: Uuid, item: &PersonalItem, cx: &mut Cx) -> AnyElement {
    let id = item.content.id;
    let active = item.state.is_active();
    let enabled = !matches!(item.state, PersonalState::Disabled);

    div()
        .id(SharedString::from(format!("installed-{id}")))
        .p(px(12.))
        .rounded(px(R_SM))
        .bg(rgba(0xffffff0a))
        .border_1()
        .border_color(rgb(if active { BORDER } else { 0x3a2a3f }))
        .flex()
        .items_center()
        .gap(px(12.))
        .child(mod_avatar(ui, &item.content.icon_url))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(3.))
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
                                .text_color(rgb(if active { TEXT_PRIMARY } else { TEXT_MUTED }))
                                .truncate()
                                .child(item.content.title.clone()),
                        )
                        .when(!item.content.version_name.is_empty(), |d| {
                            d.child(
                                div()
                                    .flex_shrink_0()
                                    .text_size(px(10.))
                                    .text_color(rgb(TEXT_MUTED))
                                    .child(item.content.version_name.clone()),
                            )
                        }),
                )
                .child(state_line(item)),
        )
        .child(actions(
            server_id,
            id,
            enabled,
            ui.is_armed(&format!("remove-{id}")),
            cx,
        ))
        .into_any_element()
}

/// Why it is or is not running, in the player's own language.
fn state_line(item: &PersonalItem) -> AnyElement {
    let (text, colour) = match &item.state {
        PersonalState::Active => (item.content.path.clone(), TEXT_MUTED),
        PersonalState::Disabled => (t("content-state-disabled"), TEXT_MUTED),
        PersonalState::Superseded { by } => {
            let mut args = i18n::FluentArgs::new();
            args.set("file", by.clone());
            (i18n::t_args("content-state-superseded", &args), BLUE)
        }
        PersonalState::Blocked { reason } => {
            let mut args = i18n::FluentArgs::new();
            args.set("reason", reason.clone());
            (i18n::t_args("content-state-blocked", &args), ERROR)
        }
        PersonalState::Incompatible { mc_version, loader } => {
            let mut args = i18n::FluentArgs::new();
            args.set("mc", mc_version.clone());
            args.set("loader", loader.clone());
            (i18n::t_args("content-state-incompatible", &args), WARNING)
        }
    };

    div()
        .text_size(px(11.))
        .text_color(rgb(colour))
        .truncate()
        .child(text)
        .into_any_element()
}

fn actions(server_id: Uuid, id: Uuid, enabled: bool, armed: bool, cx: &mut Cx) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .child(icon_action(
            format!("toggle-{id}"),
            if enabled { "eye" } else { "eye-off" },
            if enabled { TEXT_MUTED } else { WARNING },
            t(if enabled {
                "hint-content-off"
            } else {
                "hint-content-on"
            }),
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                // One change at a time: a second click while the first is on its
                // way used to send it twice.
                if this.content_busy {
                    return;
                }
                this.content_busy = true;
                this.backend
                    .send(MessageToBackend::SetPersonalContentEnabled {
                        server_id,
                        id,
                        enabled: !enabled,
                    });
                cx.notify();
            }),
        ))
        // Removing takes two clicks: the second says what it will do.
        .child(if armed {
            div()
                .id(SharedString::from(format!("remove-{id}")))
                .h(px(30.))
                .px(px(10.))
                .rounded(px(R_SM))
                .flex()
                .items_center()
                .cursor_pointer()
                .bg(rgb(ERROR))
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .text_color(rgb(TEXT_PRIMARY))
                .child(t("common-delete"))
                .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    if this.confirm_or_arm(format!("remove-{id}"), cx) {
                        // One change at a time: a second click while the first is on its
                        // way used to send it twice.
                        if this.content_busy {
                            return;
                        }
                        this.content_busy = true;
                        this.backend
                            .send(MessageToBackend::RemovePersonalContent { server_id, id });
                    }
                    cx.notify();
                }))
                .into_any_element()
        } else {
            icon_action(
                format!("remove-{id}"),
                "trash-2",
                ERROR,
                t("common-delete"),
                cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                    this.confirm_or_arm(format!("remove-{id}"), cx);
                    cx.notify();
                }),
            )
        })
        .into_any_element()
}

fn icon_action(
    id: String,
    icon: &'static str,
    colour: u32,
    hint: String,
    on_click: impl Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> AnyElement {
    div()
        .id(SharedString::from(id))
        .tooltip(crate::components::hint(hint))
        .size(px(30.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|d| d.bg(rgba(0xffffff12)))
        .child(ic(icon, 15., colour))
        .on_click(on_click)
        .into_any_element()
}
