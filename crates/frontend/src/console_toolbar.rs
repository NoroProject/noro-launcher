// Over 150 lines: one toolbar; its filters, search, copy, find and save act on
// the same buffer.
use crate::console_controls::{action, clipboard_text, toggle};
use crate::console_model::Filters;
use crate::state::{ConsoleWindow, GlobalLauncherUI};
use crate::theme::*;
use gpui::{
    div, prelude::*, px, rgb, AnyElement, AsyncApp, ClipboardItem, Context, FontWeight, WeakEntity,
};
use i18n::t;

type Cx<'a> = Context<'a, ConsoleWindow>;

/// The level toggles keep log4j's own names: they are what the lines say.
pub fn toolbar(view: &ConsoleWindow, cx: &mut Cx) -> AnyElement {
    let filters = view.buffer.filters().clone();
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
            view.buffer.visible_len(),
            view.buffer.total(),
            &filters.query,
            &view.status_message,
        ))
        .child(div().flex_1())
        .child(toggle(
            "INFO",
            filters.info,
            |v| {
                let f = v.buffer.filters().clone();
                v.set_filters(Filters { info: !f.info, ..f });
            },
            cx,
        ))
        .child(toggle(
            "WARN",
            filters.warn,
            |v| {
                let f = v.buffer.filters().clone();
                v.set_filters(Filters { warn: !f.warn, ..f });
            },
            cx,
        ))
        .child(toggle(
            "ERROR",
            filters.error,
            |v| {
                let f = v.buffer.filters().clone();
                v.set_filters(Filters {
                    error: !f.error,
                    ..f
                });
            },
            cx,
        ))
        .child(action(
            "console-bottom",
            t("console-to-bottom"),
            |v, _| {
                v.follow();
                v.status_message.clear();
            },
            cx,
        ))
        .child(action(
            "console-copy",
            if view.copy_success {
                format!("✓ {}", t("common-copied").to_uppercase())
            } else {
                t("common-copy").to_uppercase()
            },
            |v, cx| {
                // Built on the click, not on every frame the button is drawn.
                let text = v.buffer.copy_text();
                let lines = v.buffer.visible_len();
                cx.write_to_clipboard(ClipboardItem::new_string(text));
                v.status_message = i18n::t_count("console-copied", lines as i64);
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
            "console-find",
            t("console-find"),
            |v, cx| {
                // GPUI has no text field; the search text comes from the
                // clipboard.
                let query = clipboard_text(cx)
                    .map(|q| q.trim().to_lowercase())
                    .unwrap_or_default();
                if query.is_empty() {
                    v.status_message = t("console-clipboard-empty");
                    return;
                }
                v.status_message.clear();
                let f = v.buffer.filters().clone();
                v.set_filters(Filters { query, ..f });
            },
            cx,
        ))
        .child(action(
            "console-reset",
            t("console-reset"),
            |v, _| {
                v.status_message.clear();
                v.set_filters(Filters::default());
            },
            cx,
        ))
        .child(action("console-save", t("console-save"), save, cx))
        .child(action(
            "console-clear",
            if view.clear_armed {
                t("console-clear-confirm")
            } else {
                t("console-clear")
            },
            |v, cx| {
                // Clearing can't be undone and the line that mattered is
                // usually the one just scrolled past: ask once.
                if !v.clear_armed {
                    v.clear_armed = true;
                    disarm_later(cx);
                    return;
                }
                v.clear_armed = false;
                let count = v.buffer.total();
                let server_id = v.server_id;
                v.show_server(server_id, Vec::new());
                v.status_message = i18n::t_count("console-cleared", count as i64);
                if let Some(ui) = cx.try_global::<GlobalLauncherUI>() {
                    let ui = ui.0.clone();
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

fn disarm_later(cx: &mut Cx) {
    cx.spawn(|view: WeakEntity<ConsoleWindow>, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(3))
                .await;
            let _ = view.update(&mut cx, |v, cx| {
                v.clear_armed = false;
                cx.notify();
            });
        }
    })
    .detach();
}

/// The visible lines to a file the player picks. Copying thousands of lines
/// through the clipboard into a chat is how logs used to reach support.
fn save(v: &mut ConsoleWindow, cx: &mut Cx) {
    let text = v.buffer.copy_text();
    if text.is_empty() {
        return;
    }
    let name = format!(
        "noro-console-{}.log",
        chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
    );
    let dir = dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(std::env::temp_dir);
    let picked = cx.prompt_for_new_path(&dir, Some(&name));
    cx.spawn(|view: WeakEntity<ConsoleWindow>, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            let Ok(Ok(Some(path))) = picked.await else {
                return; // cancelled
            };
            let written = cx
                .background_executor()
                .spawn(async move { std::fs::write(&path, text) })
                .await;
            let _ = view.update(&mut cx, |v, cx| {
                v.status_message = match written {
                    Ok(()) => t("console-saved"),
                    Err(e) => format!("{}: {e}", t("console-save-failed")),
                };
                cx.notify();
            });
        }
    })
    .detach();
}

fn title(visible: usize, total: usize, query: &str, status: &str) -> AnyElement {
    let mut suffix = if query.is_empty() {
        format!("{visible}/{total}")
    } else {
        let mut args = i18n::FluentArgs::new();
        args.set("query", query.to_string());
        format!(
            "{visible}/{total} {}",
            i18n::t_args("console-find-query", &args)
        )
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
