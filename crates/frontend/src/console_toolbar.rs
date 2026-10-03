//! The console's toolbar: search, the level filters, and what to do with the
//! log — follow it, copy it, clear it, change how it looks.

use crate::console_controls::{icon_button, level_chip, search_field};
use crate::console_window::ConsoleWindow;
use crate::state::GlobalLauncherUI;
use crate::theme::*;
use gpui::{div, prelude::*, px, rgb, AnyElement, AsyncApp, ClipboardItem, Context, WeakEntity};

type Cx<'a> = Context<'a, ConsoleWindow>;

pub fn toolbar(view: &mut ConsoleWindow, cx: &mut Cx) -> AnyElement {
    let counts = view.counts();
    let s = view.settings;
    let following = view.is_following();
    let copied = view.copied;
    let settings_open = view.settings_open;
    div()
        .h(px(48.))
        .w_full()
        .flex_shrink_0()
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(8.))
        .bg(rgb(BG_PANEL))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(search_field(view, cx))
        .child(level_chip(
            "console-info",
            "INFO",
            counts[0],
            TEXT_SECONDARY,
            s.show_info,
            |s| s.show_info = !s.show_info,
            cx,
        ))
        .child(level_chip(
            "console-warn",
            "WARN",
            counts[1],
            WARNING,
            s.show_warn,
            |s| s.show_warn = !s.show_warn,
            cx,
        ))
        .child(level_chip(
            "console-error",
            "ERROR",
            counts[2],
            ERROR,
            s.show_error,
            |s| s.show_error = !s.show_error,
            cx,
        ))
        .child(div().flex_1())
        .child(icon_button(
            "console-follow",
            "arrow-down-to-line",
            following,
            |v, _| v.follow(),
            cx,
        ))
        .child(icon_button(
            "console-copy",
            if copied { "check" } else { "copy" },
            copied,
            copy,
            cx,
        ))
        .child(icon_button("console-clear", "trash-2", false, clear, cx))
        .child(icon_button(
            "console-view",
            "sliders-horizontal",
            settings_open,
            |v, _| v.settings_open = !v.settings_open,
            cx,
        ))
        .into_any_element()
}

fn copy(view: &mut ConsoleWindow, cx: &mut Cx) {
    cx.write_to_clipboard(ClipboardItem::new_string(view.shown_text()));
    view.copied = true;
    cx.spawn(|view: WeakEntity<ConsoleWindow>, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(2))
                .await;
            let _ = view.update(&mut cx, |v, cx| {
                v.copied = false;
                cx.notify();
            });
        }
    })
    .detach();
}

fn clear(view: &mut ConsoleWindow, cx: &mut Cx) {
    view.clear();
    // The launcher keeps a copy for the next console it opens; it goes too.
    if let Some(ui) = cx.try_global::<GlobalLauncherUI>() {
        let ui = ui.0.clone();
        let server_id = view.server_id;
        ui.update(cx, |ui, cx| {
            ui.logs.remove(&server_id);
            cx.notify();
        });
    }
}
