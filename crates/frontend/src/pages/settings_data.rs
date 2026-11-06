// Over 150 lines: the folder row and the four states under it — the
// confirmation, the progress, an interrupted move, and the last outcome.
//! The game files folder in the settings: where it is, how big, and moving it.

use super::common::{panel, Cx};
use super::settings_rows::{mono_value, row};
use crate::components::{badge, btn, progress_bar};
use crate::state::{data_text, size_text, LauncherUI};
use crate::theme::*;
use bridge::{DataLocationView, DataMovePhase, DataMovePlan, DataMoveProgress, DataStatus};
use gpui::{div, prelude::*, px, rgb, AnyElement, SharedString};
use i18n::{t, t_args, FluentArgs};
use schema::NotifLevel;

pub fn data_panel(ui: &LauncherUI, cx: &mut Cx) -> AnyElement {
    // Comes at start; nothing to show for the instant before it.
    let Some(view) = ui.data.view.clone() else {
        return div().into_any_element();
    };
    let below = div()
        .px(px(20.))
        .pb(px(16.))
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(path_line(&view))
        .child(size_line(ui, &view))
        .when_some(ui.data.plan.clone(), |d, plan| {
            d.child(plan_block(&plan, cx))
        })
        .when_some(ui.data.progress.clone(), |d, p| {
            d.child(progress_block(ui, &p))
        })
        .when_some(interrupted(&view.status), |d, (from, to)| {
            d.child(interrupted_block(&from, &to, cx))
        })
        .when_some(ui.data.note.clone(), |d, (text, level)| {
            d.child(line(data_text(&text), note_color(level)))
        });
    panel()
        .flex()
        .flex_col()
        .child(row(
            "folder",
            t("settings-data"),
            t("settings-data-hint"),
            controls(ui, &view, cx),
            false,
        ))
        .child(below)
        .into_any_element()
}

fn interrupted(status: &DataStatus) -> Option<(String, String)> {
    match status {
        DataStatus::Interrupted { from, to } => {
            Some((from.display().to_string(), to.display().to_string()))
        }
        _ => None,
    }
}

fn controls(ui: &LauncherUI, view: &DataLocationView, cx: &mut Cx) -> AnyElement {
    let buttons = div().flex().items_center().gap(px(8.));
    if ui.data.moving() {
        return buttons
            .child(btn(
                "data-cancel-move",
                t("settings-data-cancel"),
                false,
                cx.listener(|this, _e, _w, cx| {
                    this.cancel_data_move();
                    cx.notify();
                }),
            ))
            .into_any_element();
    }
    if view.status != DataStatus::Ready {
        return buttons.into_any_element();
    }
    buttons
        .child(btn(
            "data-open",
            t("settings-data-open"),
            false,
            cx.listener(|this, _e, _w, _cx| this.open_data_folder()),
        ))
        .when(!view.is_default, |d| {
            d.child(btn(
                "data-move-back",
                t("settings-data-move-back"),
                false,
                cx.listener(|this, _e, _w, cx| {
                    this.move_data_back();
                    cx.notify();
                }),
            ))
        })
        .child(btn(
            "data-move",
            t("settings-data-move"),
            true,
            cx.listener(|this, _e, _w, cx| {
                this.pick_data_folder(cx);
                cx.notify();
            }),
        ))
        .into_any_element()
}

fn path_line(view: &DataLocationView) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .child(mono_value(&view.path.display().to_string(), ""))
        .when(view.is_default, |d| {
            d.child(badge(t("settings-data-default"), TEXT_MUTED))
        })
        .into_any_element()
}

fn size_line(ui: &LauncherUI, view: &DataLocationView) -> AnyElement {
    let text = match (&view.status, view.size_bytes) {
        (DataStatus::Unavailable, _) => return line(t("settings-data-unavailable"), WARNING),
        (_, Some(size)) => {
            let mut args = FluentArgs::new();
            args.set("size", size_text(size));
            args.set(
                "free",
                view.free_bytes.map(size_text).unwrap_or_else(|| "?".into()),
            );
            t_args("settings-data-size", &args)
        }
        // Not asked yet, or still counting.
        (_, None) if ui.data.counting || ui.data.moving() => t("settings-data-counting"),
        (_, None) => String::new(),
    };
    line(text, TEXT_MUTED)
}

fn plan_block(plan: &DataMovePlan, cx: &mut Cx) -> AnyElement {
    let mut args = FluentArgs::new();
    args.set("size", size_text(plan.bytes));
    args.set("path", plan.target.display().to_string());
    let how = if plan.same_volume {
        t("settings-data-plan-rename")
    } else {
        let mut free = FluentArgs::new();
        free.set(
            "free",
            plan.free.map(size_text).unwrap_or_else(|| "?".into()),
        );
        t_args("settings-data-plan-copy", &free)
    };
    div()
        .p(px(12.))
        .rounded(px(R_SM))
        .border_1()
        .border_color(rgb(BORDER))
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(line(t_args("settings-data-plan", &args), TEXT_PRIMARY))
        .child(line(how, TEXT_MUTED))
        .child(
            div()
                .flex()
                .gap(px(8.))
                .child(btn(
                    "data-start",
                    t("settings-data-start"),
                    true,
                    cx.listener(|this, _e, _w, cx| {
                        this.start_data_move();
                        cx.notify();
                    }),
                ))
                .child(btn(
                    "data-plan-cancel",
                    t("settings-data-cancel"),
                    false,
                    cx.listener(|this, _e, _w, cx| {
                        this.cancel_data_plan();
                        cx.notify();
                    }),
                )),
        )
        .into_any_element()
}

fn progress_block(ui: &LauncherUI, p: &DataMoveProgress) -> AnyElement {
    let phase = match p.phase {
        DataMovePhase::Copying => t("settings-data-phase-copying"),
        DataMovePhase::Verifying => t("settings-data-phase-verifying"),
        DataMovePhase::Switching => t("settings-data-phase-switching"),
        DataMovePhase::Cleaning => t("settings-data-phase-cleaning"),
    };
    let counted = p.total_bytes > 0;
    let mut details = Vec::new();
    if counted {
        details.push(crate::sync_text::megabytes(p.done_bytes, p.total_bytes));
        let mut files = FluentArgs::new();
        files.set("done", p.done_files);
        files.set("total", p.total_files);
        details.push(t_args("settings-data-files", &files));
        details.extend(crate::sync_text::rate_label(
            &ui.data.rate,
            p.done_bytes,
            p.total_bytes,
        ));
    }
    let fraction = if counted {
        p.done_bytes as f32 / p.total_bytes as f32
    } else {
        1.0
    };
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(line(phase, TEXT_PRIMARY))
        .child(progress_bar(fraction))
        .when(counted, |d| d.child(line(details.join(" · "), TEXT_MUTED)))
        .into_any_element()
}

fn interrupted_block(from: &str, to: &str, cx: &mut Cx) -> AnyElement {
    let mut args = FluentArgs::new();
    args.set("from", from.to_string());
    args.set("to", to.to_string());
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(line(t_args("data-interrupted-body", &args), WARNING))
        .child(line(
            t_args("data-interrupted-rollback-hint", &args),
            TEXT_MUTED,
        ))
        .child(
            div()
                .flex()
                .gap(px(8.))
                .child(btn(
                    "data-resume",
                    t("data-interrupted-resume"),
                    true,
                    cx.listener(|this, _e, _w, cx| {
                        this.resolve_data_move(true);
                        cx.notify();
                    }),
                ))
                .child(btn(
                    "data-rollback",
                    t("data-interrupted-rollback"),
                    false,
                    cx.listener(|this, _e, _w, cx| {
                        this.resolve_data_move(false);
                        cx.notify();
                    }),
                )),
        )
        .into_any_element()
}

fn note_color(level: NotifLevel) -> u32 {
    match level {
        NotifLevel::Error => ERROR,
        NotifLevel::Warning => WARNING,
        _ => TEXT_MUTED,
    }
}

fn line(text: impl Into<SharedString>, color: u32) -> AnyElement {
    div()
        .font_family(FONT_PIXEL_ALT)
        .text_size(px(12.))
        .text_color(rgb(color))
        .child(text.into())
        .into_any_element()
}
