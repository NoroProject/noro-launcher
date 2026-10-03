// Over 150 lines: the page and its rows; each row is short and the panel lists
// them.
//! Per-server client settings: JVM, console, flags.
use super::common::{panel, tabs, Cx};
use super::server_settings_header::{open_folder_button, page_header};
use super::server_settings_memory::memory;
use crate::components::checkbox_row;
use crate::icons::ic;
use crate::state::LauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, rgba, AnyElement, ClickEvent, FontWeight, SharedString};
use i18n::t;
use uuid::Uuid;

pub fn page(ui: &mut LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    // The runtime list lives on the master and costs it a request to Mojang, so
    // it is asked for once per visit, not every frame.
    if let std::collections::hash_map::Entry::Vacant(slot) = ui.java_options.entry(server_id) {
        slot.insert(Vec::new());
        ui.backend
            .send(bridge::MessageToBackend::RequestJavaRuntimes { server_id });
    }
    let source = settings_source(ui, server_id);
    let server_name = ui
        .server(&server_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| t("server-unnamed"));
    div()
        .size_full()
        .relative()
        .bg(rgb(CONTENT_FALLBACK))
        .child(tabs(ui, cx))
        .child(
            div()
                .absolute()
                .top(px(92.))
                .left(px(32.))
                .right(px(32.))
                .bottom(px(32.))
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(page_header(
                    server_id,
                    server_name,
                    source,
                    ui.is_armed(&format!("reset-{server_id}")),
                    cx,
                ))
                .child(settings_panel(ui, server_id, cx)),
        )
        // The runtime list is drawn here rather than inside its own row: GPUI
        // lays elements out in tree order, and the "Folder" row landed on top of
        // the open list.
        .children(super::java_picker::dialog(ui, server_id, cx))
        .children(super::jvm_flags::dialog(ui, server_id, cx))
        .into_any_element()
}

/// Returns the translation key rather than the text: callers compare on it, and
/// the current language must not change the outcome.
fn settings_source(ui: &LauncherUI, server_id: Uuid) -> &'static str {
    if ui.has_server_client_override(server_id) {
        "settings-source-override"
    } else if ui.server_recommendations.contains_key(&server_id) {
        "settings-source-recommended"
    } else {
        "settings-source-default"
    }
}

fn settings_panel(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    panel()
        .overflow_hidden()
        .flex()
        .flex_col()
        .child(setting_row(
            "memory-stick",
            t("settings-memory"),
            t("settings-memory-hint"),
            memory(ui, server_id, cx),
            true,
        ))
        .child(setting_row(
            "terminal",
            t("settings-console"),
            t("settings-console-hint"),
            console(ui, server_id, cx),
            true,
        ))
        .child(setting_row(
            "maximize",
            t("settings-fullscreen"),
            t("settings-fullscreen-hint"),
            fullscreen(ui, server_id, cx),
            true,
        ))
        .child(setting_row(
            "code",
            t("settings-jvm-flags"),
            t("settings-jvm-hint"),
            super::jvm_flags::control(ui, server_id, cx),
            true,
        ))
        .child(setting_row(
            "coffee",
            t("java-title"),
            t("java-hint"),
            super::java_picker::control(ui, server_id, cx),
            true,
        ))
        .child(setting_row(
            "folder",
            t("settings-folder"),
            t("settings-folder-hint"),
            folder(ui, server_id, cx),
            false,
        ))
        .into_any_element()
}

fn setting_row(
    icon: &'static str,
    title: impl Into<gpui::SharedString>,
    subtitle: impl Into<gpui::SharedString>,
    control: AnyElement,
    border: bool,
) -> AnyElement {
    div()
        .min_h(px(72.))
        .px(px(20.))
        .py(px(14.))
        .when(border, |d| d.border_b_1().border_color(rgb(BORDER)))
        .flex()
        .items_center()
        .gap(px(16.))
        .child(
            div()
                .w(px(32.))
                .h(px(32.))
                .flex_none()
                .rounded(px(R_SM))
                .bg(rgba(0xffffff0c))
                .border_1()
                .border_color(rgb(BORDER))
                .flex()
                .items_center()
                .justify_center()
                .child(ic(icon, 16., TEXT_SECONDARY)),
        )
        .child(row_label(title, subtitle))
        .child(div().flex_1())
        .child(control)
        .into_any_element()
}

fn row_label(
    title: impl Into<gpui::SharedString>,
    subtitle: impl Into<gpui::SharedString>,
) -> AnyElement {
    let (title, subtitle) = (title.into(), subtitle.into());
    div()
        .w(px(300.))
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(11.))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(TEXT_SECONDARY))
                .child(title),
        )
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_MUTED))
                .child(subtitle),
        )
        .into_any_element()
}

fn console(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let enabled = ui.server_client_settings(server_id).show_console_on_launch;
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(checkbox_row(
            SharedString::from(format!("cs-console-on-launch-{server_id}")),
            enabled,
            true,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                let v = this
                    .server_client_settings(server_id)
                    .show_console_on_launch;
                this.set_server_show_console_on_launch(server_id, !v);
                cx.notify();
            }),
        ))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(t("settings-console-open")),
        )
        .into_any_element()
}

fn fullscreen(ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    let enabled = ui.server_client_settings(server_id).fullscreen;
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(checkbox_row(
            SharedString::from(format!("cs-fullscreen-{server_id}")),
            enabled,
            true,
            cx.listener(move |this, _e: &ClickEvent, _w, cx| {
                let v = this.server_client_settings(server_id).fullscreen;
                this.set_server_fullscreen(server_id, !v);
                cx.notify();
            }),
        ))
        .child(
            div()
                .font_family(FONT_PIXEL_ALT)
                .text_size(px(12.))
                .text_color(rgb(TEXT_SECONDARY))
                .child(t("settings-fullscreen-show")),
        )
        .into_any_element()
}
fn folder(_ui: &LauncherUI, server_id: Uuid, cx: &mut Cx) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(open_folder_button(server_id, cx))
        .into_any_element()
}
