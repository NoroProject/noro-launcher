use crate::console_controls::{action, clipboard_text, toggle};
use crate::console_model::joined_lines;
use crate::state::{ConsoleWindow, GlobalLauncherUI, LogEntry};
use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, AnyElement, AsyncApp, ClipboardItem, Context, FontWeight,
    ListAlignment, ListState, WeakEntity,
};
use i18n::t;

type Cx<'a> = Context<'a, ConsoleWindow>;

pub fn toolbar(view: &ConsoleWindow, logs: &[LogEntry], cx: &mut Cx) -> AnyElement {
    let copy_text = joined_lines(logs);
    div()
        .h(px(56.))
        .px(px(16.))
        .flex()
        .items_center()
        .gap(px(12.))
        .overflow_x_hidden()
        .bg(rgb(BG_PANEL))
        .border_b_1()
        .border_color(rgb(BORDER))
        .child(title(
            logs.len(),
            view.logs.len(),
            &view.search_query,
            &view.status_message,
        ))
        .child(div().flex_1())
        .child(toggle(
            "INFO",
            view.show_info,
            |v| v.show_info = !v.show_info,
            cx,
        ))
        .child(toggle(
            "WARN",
            view.show_warn,
            |v| v.show_warn = !v.show_warn,
            cx,
        ))
        .child(toggle(
            "ERROR",
            view.show_error,
            |v| v.show_error = !v.show_error,
            cx,
        ))
        .child(action(
            "BOTTOM",
            |v, _| {
                v.list_state = ListState::new(v.logs.len(), ListAlignment::Bottom, px(100.));
                v.status_message = "FOLLOWING".to_string();
            },
            cx,
        ))
        .child(action(
            if view.copy_success {
                "✓ COPIED!"
            } else {
                "COPY"
            },
            move |v, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(copy_text.clone()));
                let lines = copy_text.lines().count();
                v.status_message = format!("COPIED {lines}");
                v.copy_success = true;

                cx.spawn(|view: WeakEntity<ConsoleWindow>, cx: &mut AsyncApp| {
                    let mut cx = cx.clone();
                    async move {
                        cx.background_executor()
                            .timer(std::time::Duration::from_secs(2))
                            .await;
                        let _ = view.update(&mut cx, |v, cx| {
                            v.copy_success = false;
                            cx.notify();
                        });
                    }
                })
                .detach();
            },
            cx,
        ))
        .child(action(
            "FIND",
            |v, cx| {
                if let Some(query) = clipboard_text(cx) {
                    v.search_query = query.trim().to_string();
                    v.status_message = if v.search_query.is_empty() {
                        "EMPTY FIND".to_string()
                    } else {
                        "FIND FROM CLIPBOARD".to_string()
                    };
                } else {
                    v.status_message = "NO CLIPBOARD TEXT".to_string();
                }
            },
            cx,
        ))
        .child(action(
            "RESET",
            |v, _| {
                v.search_query.clear();
                v.status_message = "FILTER RESET".to_string();
            },
            cx,
        ))
        .child(action(
            "CLEAR",
            |v, cx| {
                let count = v.logs.len();
                v.logs.clear();
                v.list_state = ListState::new(0, ListAlignment::Bottom, px(100.));
                v.status_message = format!("CLEARED {count}");
                if let Some(ui) = cx.try_global::<GlobalLauncherUI>() {
                    let ui = ui.0.clone();
                    let server_id = v.server_id;
                    ui.update(cx, |ui, cx| {
                        ui.logs.remove(&server_id);
                        cx.notify();
                    });
                }
            },
            cx,
        ))
        .into_any_element()
}

fn title(visible: usize, total: usize, query: &str, status: &str) -> AnyElement {
    let mut suffix = if query.is_empty() {
        format!("{visible}/{total}")
    } else {
        format!("{visible}/{total} FIND {query}")
    };
    if !status.is_empty() {
        suffix.push_str("  ");
        suffix.push_str(status);
    }

    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .min_w_0()
        .font_family(FONT_PIXEL_ALT)
        .text_color(rgb(TEXT_SECONDARY))
        .child(
            div()
                .text_size(px(16.))
                .font_weight(FontWeight::BOLD)
                .child(t("console-title")),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(rgb(TEXT_MUTED))
                .overflow_hidden()
                .text_ellipsis()
                .child(suffix),
        )
        .into_any_element()
}
