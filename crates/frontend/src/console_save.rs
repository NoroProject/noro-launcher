//! Saving the console to a file.

use crate::console_toolbar::after_a_moment;
use crate::console_window::ConsoleWindow;
use gpui::{AsyncApp, Context, WeakEntity};

/// The shown lines to a file the player picks. Copying thousands of lines
/// through the clipboard into a chat is how logs used to reach support.
pub fn save(view: &mut ConsoleWindow, cx: &mut Context<ConsoleWindow>) {
    let text = view.shown_text();
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
                if let Err(e) = &written {
                    tracing::warn!("could not save the console: {e}");
                }
                v.saved = Some(written.is_ok());
                after_a_moment(cx, |v| v.saved = None);
                cx.notify();
            });
        }
    })
    .detach();
}
