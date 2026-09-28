// Over 150 lines: a closed select, its open list and one row of it. Split, a
// row has nowhere to say what picking it does.
//! Choosing which Java runtime the build launches on.
//!
//! The list comes from the master, which holds Mojang's own runtimes for every
//! platform and hands out the one that was picked. Nothing local is offered: a
//! JDK found on the player's machine is an unknown build of an unknown version,
//! and a crash on it is indistinguishable from a broken mod for an evening.
//!
//! A select rather than a row of chips. There are five or six runtimes, only
//! one is in force, and chips spread that one choice over three lines while
//! shouting about the five that were not taken.
//!
//! The list opens as a dialog rather than a drop-down under the field. GPUI
//! paints in tree order, so a panel positioned inside its own settings row ends
//! up under every row that comes after it — the folder row with its button drew
//! straight over the options.

use super::common::Cx;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use bridge::MessageToBackend;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use i18n::t;
use schema::java::JavaRuntimeOption;
use uuid::Uuid;

/// The control that goes into the settings row.
pub fn control(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    if ui.java_busy {
        return div()
            .flex()
            .items_center()
            .gap(px(8.))
            .child(ic("refresh", 13., WARNING))
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(rgb(WARNING))
                    .child(t("java-downloading")),
            )
            .into_any_element();
    }

    let options = ui.java_options.get(&server_id).cloned().unwrap_or_default();
    let selected = ui.java_selected.get(&server_id).cloned().flatten();
    let default_component = ui.java_default.get(&server_id).cloned().unwrap_or_default();

    closed(ui, server_id, &options, &selected, &default_component, cx)
}

/// The open list, drawn at page level so nothing can cover it.
pub fn dialog(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> Option<AnyElement> {
    if !ui.java_picker_open {
        return None;
    }
    let options = ui.java_options.get(&server_id).cloned().unwrap_or_default();
    let selected = ui.java_selected.get(&server_id).cloned().flatten();
    let default_component = ui.java_default.get(&server_id).cloned().unwrap_or_default();

    Some(
        div()
            .id("java-backdrop")
            .absolute()
            .inset_0()
            .bg(rgba(0x000000aa))
            .flex()
            .items_center()
            .justify_center()
            .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                this.java_picker_open = false;
                cx.notify();
            }))
            .child(open_list(
                &options,
                &selected,
                &default_component,
                server_id,
                cx,
            ))
            .into_any_element(),
    )
}

/// What the runtime is called in one line.
fn label(option: &JavaRuntimeOption, default_component: &str) -> String {
    let mut args = i18n::FluentArgs::new();
    args.set("major", option.major.to_string());
    let mut text = i18n::t_args("java-version", &args);
    if !option.version.is_empty() {
        text = format!("{text} · {}", option.version);
    }
    if option.component == default_component {
        text = format!("{text} · {}", t("java-recommended"));
    }
    text
}

fn closed(
    ui: &LauncherUI,
    server_id: Uuid,
    options: &[JavaRuntimeOption],
    selected: &Option<String>,
    default_component: &str,
    cx: &mut Cx,
) -> AnyElement {
    let current = match selected {
        None => t("java-default"),
        Some(component) => options
            .iter()
            .find(|o| &o.component == component)
            .map(|o| label(o, default_component))
            // Picked a runtime the list no longer offers: name it rather than
            // showing "build default", which would be a different setting.
            .unwrap_or_else(|| component.clone()),
    };
    let empty = options.is_empty();

    div()
        .id("java-select")
        .w(px(280.))
        .h(px(36.))
        .px(px(12.))
        .rounded(px(R_SM))
        .bg(rgb(BG_INPUT))
        .border_1()
        .border_color(rgb(if ui.java_picker_open { ACCENT } else { BORDER }))
        .flex()
        .items_center()
        .gap(px(8.))
        .cursor_pointer()
        .hover(|d| d.border_color(rgb(CTA)))
        .child(ic("coffee", 14., TEXT_MUTED))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(12.))
                .text_color(rgb(if selected.is_some() {
                    CTA
                } else {
                    TEXT_SECONDARY
                }))
                .truncate()
                .child(current),
        )
        .child(ic(
            if ui.java_picker_open {
                "chevron-up"
            } else {
                "chevron-down"
            },
            13.,
            TEXT_MUTED,
        ))
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.java_picker_open = !this.java_picker_open;
            // The list costs a request to Mojang, so it is fetched when it is
            // actually going to be looked at.
            if this.java_picker_open && empty {
                this.backend
                    .send(MessageToBackend::RequestJavaRuntimes { server_id });
            }
            cx.notify();
        }))
        .into_any_element()
}

fn open_list(
    options: &[JavaRuntimeOption],
    selected: &Option<String>,
    default_component: &str,
    server_id: Uuid,
    cx: &mut Cx,
) -> AnyElement {
    let mut rows: Vec<AnyElement> = vec![row(
        "java-opt-default".to_string(),
        t("java-default"),
        selected.is_none(),
        server_id,
        None,
        cx,
    )];
    rows.extend(options.iter().map(|o| {
        row(
            format!("java-opt-{}", o.component),
            label(o, default_component),
            selected.as_deref() == Some(o.component.as_str()),
            server_id,
            Some(o.component.clone()),
            cx,
        )
    }));

    div()
        .id("java-dialog")
        .occlude()
        .on_click(|_, _, _| {})
        .w(px(420.))
        .max_h(px(420.))
        .flex()
        .flex_col()
        .rounded(px(R_LG))
        .bg(rgb(BG_PANEL))
        .border_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .h(px(48.))
                .px(px(16.))
                .flex()
                .items_center()
                .gap(px(8.))
                .border_b_1()
                .border_color(rgb(BORDER))
                .child(ic("coffee", 15., ACCENT))
                .child(
                    div()
                        .flex_1()
                        .font_family(FONT_PIXEL_ALT)
                        .text_size(px(13.))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(CTA))
                        .child(t("java-title")),
                )
                .child(
                    div()
                        .id("java-close")
                        .size(px(28.))
                        .rounded(px(R_SM))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|d| d.bg(rgba(0xffffff10)))
                        .child(ic("x", 14., TEXT_MUTED))
                        .on_click(cx.listener(|this, _e: &ClickEvent, _w, cx| {
                            this.java_picker_open = false;
                            cx.notify();
                        })),
                ),
        )
        .child(
            div()
                .id("java-options")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .p(px(8.))
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(rows),
        )
        .into_any_element()
}

fn row(
    id: String,
    text: String,
    active: bool,
    server_id: Uuid,
    component: Option<String>,
    cx: &mut Cx,
) -> AnyElement {
    div()
        .id(SharedString::from(id))
        .px(px(10.))
        .h(px(32.))
        .rounded(px(R_SM))
        .flex()
        .items_center()
        .gap(px(8.))
        .cursor_pointer()
        .bg(if active {
            rgba((CTA << 8) | 0x16)
        } else {
            rgba(0x00000000)
        })
        .hover(|d| d.bg(rgba(0xffffff10)))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(12.))
                .font_weight(if active {
                    FontWeight::BOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(rgb(if active { CTA } else { TEXT_SECONDARY }))
                .truncate()
                .child(text),
        )
        .when(active, |d| d.child(ic("check", 13., CTA)))
        .on_click(cx.listener(move |this, _e: &ClickEvent, _w, cx| {
            this.java_picker_open = false;
            // Picking makes the master download the runtime — hundreds of
            // megabytes the first time. The row says so meanwhile.
            this.java_busy = true;
            this.content_error = None;
            this.backend.send(MessageToBackend::SetJavaRuntime {
                server_id,
                component: component.clone(),
            });
            cx.notify();
        }))
        .into_any_element()
}
