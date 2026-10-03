//! The global settings panel and its controls.

use super::common::{panel, Cx};
use super::settings_rows::{mono_value, row, stepper};
use crate::components::{btn, checkbox_row};
use crate::state::LauncherUI;
use gpui::{div, prelude::*, px, AnyElement, ClickEvent, SharedString};
use i18n::t;

pub fn settings_panel(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    panel()
        .flex()
        .flex_col()
        .child(row(
            "memory-stick",
            t("settings-memory-default"),
            t("settings-memory-hint"),
            memory_control(ui, cx),
            true,
        ))
        .child(row(
            "terminal",
            t("settings-console"),
            t("settings-console-hint"),
            console_control(ui, cx),
            true,
        ))
        .child(row(
            "maximize",
            t("settings-fullscreen"),
            t("settings-fullscreen-hint"),
            fullscreen_control(ui, cx),
            true,
        ))
        .child(row(
            "user",
            t("settings-discord-rpc"),
            t("settings-discord-rpc-hint"),
            checkbox_row(
                SharedString::new_static("g-discord-rpc"),
                ui.config.discord_rpc,
                true,
                cx.listener(|this, _e: &ClickEvent, _w, cx| {
                    let v = this.config.discord_rpc;
                    this.set_discord_rpc(!v);
                    cx.notify();
                }),
            )
            .into_any_element(),
            true,
        ))
        .child(row(
            "code",
            t("settings-jvm-flags"),
            t("settings-jvm-hint"),
            mono_value(&ui.config.jvm_flags, &t("settings-jvm-not-set")),
            ui.config.crash_reports_available,
        ))
        // Hidden when no DSN is baked into the build: the toggle would flip
        // with nowhere to send.
        .when(ui.config.crash_reports_available, |d| {
            d.child(row(
                "triangle-alert",
                t("settings-crash-reports"),
                t("settings-crash-reports-hint"),
                crash_reports_control(ui, cx),
                true,
            ))
        })
        .child(row(
            "circle-alert",
            t("settings-support-bundle"),
            t("settings-support-bundle-hint"),
            support_bundle_control(cx),
            false,
        ))
        .into_any_element()
}

/// Player-initiated log upload: no admin request, no consent prompt.
fn support_bundle_control(cx: &mut Cx) -> AnyElement {
    btn(
        "send-support-bundle",
        t("settings-support-send"),
        false,
        cx.listener(|this, _e, _w, cx| {
            this.send_support_bundle();
            cx.notify();
        }),
    )
    .into_any_element()
}

fn crash_reports_control(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    checkbox_row(
        SharedString::new_static("g-crash-reports"),
        ui.config.crash_reports,
        true,
        cx.listener(|this, _e: &ClickEvent, _w, cx| {
            let v = this.config.crash_reports;
            this.set_crash_reports(!v);
            cx.notify();
        }),
    )
    .into_any_element()
}

fn memory_control(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    let warning = ui.memory_warning(ui.config.memory_max_mb);
    let steppers = div()
        .flex()
        .items_center()
        .gap(px(16.))
        .child(stepper(
            "g-mem-min",
            t("settings-memory-min"),
            ui.config.memory_min_mb,
            |ui, d| adjust(ui, true, d),
            cx,
        ))
        .child(stepper(
            "g-mem-max",
            t("settings-memory-max"),
            ui.config.memory_max_mb,
            |ui, d| adjust(ui, false, d),
            cx,
        ));
    div()
        .flex()
        .flex_col()
        .items_end()
        .gap(px(6.))
        .child(steppers)
        .children(warning.map(super::settings_rows::warning_line))
        .into_any_element()
}

fn console_control(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    checkbox_row(
        SharedString::new_static("g-console-on-launch"),
        ui.config.show_console_on_launch,
        true,
        cx.listener(|this, _e: &ClickEvent, _w, cx| {
            let v = this.config.show_console_on_launch;
            this.set_show_console_on_launch(!v);
            cx.notify();
        }),
    )
    .into_any_element()
}

fn fullscreen_control(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    checkbox_row(
        SharedString::new_static("g-fullscreen"),
        ui.config.fullscreen,
        true,
        cx.listener(|this, _e: &ClickEvent, _w, cx| {
            let v = this.config.fullscreen;
            this.set_fullscreen(!v);
            cx.notify();
        }),
    )
    .into_any_element()
}

/// Keeps the max at or above the min by raising the max, never by lowering the
/// min the player just set.
fn adjust(ui: &mut LauncherUI, is_min: bool, delta: i32) {
    let ceiling = ui.memory_ceiling_mb() as i32;
    let mut min = ui.config.memory_min_mb as i32;
    let mut max = ui.config.memory_max_mb as i32;
    if is_min {
        min = (min + delta).clamp(512, ceiling);
    } else {
        max = (max + delta).clamp(512, ceiling);
    }
    if max < min {
        max = min;
    }
    ui.set_memory(min as u32, max as u32);
}
