//! Панель глобальных настроек и её элементы управления.

use super::common::{panel, Cx};
use super::settings_rows::{mono_value, row, stepper};
use crate::components::checkbox_row;
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
            "code",
            t("settings-jvm-flags"),
            t("settings-jvm-hint"),
            mono_value(&ui.config.jvm_flags, "not set"),
            ui.config.crash_reports_available,
        ))
        // Строки нет, если в сборку не вшит DSN: переключать было бы нечего,
        // а сама строка обещала бы игроку то, чего не происходит.
        .when(ui.config.crash_reports_available, |d| {
            d.child(row(
                "triangle-alert",
                t("settings-crash-reports"),
                t("settings-crash-reports-hint"),
                crash_reports_control(ui, cx),
                false,
            ))
        })
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
    div()
        .flex()
        .items_center()
        .gap(px(16.))
        .child(stepper(
            "g-mem-min",
            "MIN",
            ui.config.memory_min_mb,
            |ui, d| adjust(ui, true, d),
            cx,
        ))
        .child(stepper(
            "g-mem-max",
            "MAX",
            ui.config.memory_max_mb,
            |ui, d| adjust(ui, false, d),
            cx,
        ))
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

/// Шаг памяти: минимум не выше максимума, максимум не ниже минимума.
fn adjust(ui: &mut LauncherUI, is_min: bool, delta: i32) {
    let mut min = ui.config.memory_min_mb as i32;
    let mut max = ui.config.memory_max_mb as i32;
    if is_min {
        min = (min + delta).clamp(512, 65536);
    } else {
        max = (max + delta).clamp(512, 65536);
    }
    if max < min {
        max = min;
    }
    ui.set_memory(min as u32, max as u32);
}
