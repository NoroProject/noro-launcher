//! The loading window: logo, mascot and a progress bar while core downloads.
//!
//! Drawn with the same GPUI as the launcher, so the first thing a player sees
//! looks like the rest of the interface rather than a patch bolted on.

use gpui::{
    div, img, prelude::*, px, rgb, App, Context, IntoElement, SharedString, Window, WindowOptions,
};
use parking_lot::Mutex;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

const BG: u32 = 0x0d1b2e;
const PANEL: u32 = 0x13233d;
const CREAM: u32 = 0xf3e7b3;
const MUTED: u32 = 0x5a6b91;
const BORDER: u32 = 0x223a55;

const WINDOW: (f32, f32) = (420., 320.);

#[derive(rust_embed::Embed)]
#[folder = "assets/"]
struct SplashAssets;

#[derive(Default, Clone)]
pub struct Progress {
    pub label: String,
    pub done: u64,
    /// Zero when the size isn't known; the bar then stays empty and the label
    /// carries the count.
    pub total: u64,
}

pub enum Update {
    Progress(Progress),
    /// The work stopped and waits for a [`Choice`].
    Failed {
        message: String,
        /// The technical reason, small and muted: what a player copies into a
        /// support chat.
        detail: String,
    },
}

pub type Reporter = UnboundedSender<Update>;

/// What the player picked on the error screen. Closing the window counts as
/// `Close`: the sender goes away with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Retry,
    Close,
}

pub type Choices = std::sync::mpsc::Receiver<Choice>;

struct Failure {
    message: SharedString,
    detail: SharedString,
}

struct Splash {
    progress: Progress,
    failure: Option<Failure>,
    choices: std::sync::mpsc::Sender<Choice>,
}

impl Splash {
    fn new(
        mut rx: UnboundedReceiver<Update>,
        choices: std::sync::mpsc::Sender<Choice>,
        cx: &mut Context<Self>,
    ) -> Self {
        // Redrawn on events rather than a timer: this sleeps on `recv` until
        // the download has a new number, so there are no idle frames.
        cx.spawn(async move |this, cx| {
            while let Some(next) = rx.recv().await {
                if this
                    .update(cx, |splash, cx| {
                        match next {
                            Update::Progress(p) => {
                                splash.failure = None;
                                splash.progress = p;
                            }
                            Update::Failed { message, detail } => {
                                splash.failure = Some(Failure {
                                    message: message.into(),
                                    detail: detail.into(),
                                });
                            }
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        Self {
            progress: Progress::default(),
            failure: None,
            choices,
        }
    }

    fn choose(&mut self, choice: Choice, cx: &mut Context<Self>) {
        if self.failure.take().is_some() {
            let _ = self.choices.send(choice);
            self.progress = Progress::default();
            cx.notify();
        }
    }
}

impl Render for Splash {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = div()
            .flex()
            .items_center()
            .gap(px(10.))
            .child(img("logo.png").size(px(28.)))
            .child(
                div()
                    .font_family("Monocraft")
                    .text_size(px(26.))
                    .text_color(rgb(CREAM))
                    .child(SharedString::new_static("NORO")),
            );
        let root = div()
            .size_full()
            .bg(rgb(BG))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            // Tighter on the error screen, which has two more lines.
            .gap(px(if self.failure.is_some() { 12. } else { 16. }))
            .px(px(24.))
            .child(img("mascot-loading.png").h(px(112.)).w(px(95.)))
            .child(header);

        if let Some(failure) = &self.failure {
            return root
                .child(
                    div()
                        .font_family("Monocraft")
                        .text_size(px(13.))
                        .text_color(rgb(CREAM))
                        .text_center()
                        .child(failure.message.clone()),
                )
                .child(
                    div()
                        .font_family("Monocraft")
                        .text_size(px(10.))
                        .text_color(rgb(MUTED))
                        .text_center()
                        .max_h(px(28.))
                        .overflow_hidden()
                        .child(failure.detail.clone()),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(10.))
                        .child(
                            button("retry", i18n::t("boot-retry"), true).on_click(
                                cx.listener(|this, _, _, cx| this.choose(Choice::Retry, cx)),
                            ),
                        )
                        .child(button("close", i18n::t("boot-close"), false).on_click(
                            cx.listener(|this, _, _, cx| this.choose(Choice::Close, cx)),
                        )),
                );
        }

        let p = &self.progress;
        let ratio = if p.total > 0 {
            (p.done as f32 / p.total as f32).clamp(0., 1.)
        } else {
            0.
        };
        root.child(bar(ratio)).child(
            div()
                .font_family("Monocraft")
                .text_size(px(13.))
                .text_color(rgb(MUTED))
                .child(SharedString::from(p.label.clone())),
        )
    }
}

fn button(id: &'static str, label: String, primary: bool) -> gpui::Stateful<gpui::Div> {
    let (bg, fg) = if primary { (CREAM, BG) } else { (PANEL, CREAM) };
    div()
        .id(id)
        .px(px(16.))
        .py(px(6.))
        .rounded(px(4.))
        .border_1()
        .border_color(rgb(BORDER))
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .font_family("Monocraft")
        .text_size(px(13.))
        .cursor_pointer()
        .hover(|s| s.opacity(0.85))
        .child(SharedString::from(label))
}

fn bar(ratio: f32) -> impl IntoElement {
    div()
        .w(px(280.))
        .h(px(12.))
        .rounded(px(4.))
        .bg(rgb(PANEL))
        .border_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .h_full()
                .w(px(280. * ratio))
                .rounded(px(4.))
                .bg(rgb(CREAM)),
        )
}

struct SplashAssetSource;

impl gpui::AssetSource for SplashAssetSource {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok(SplashAssets::get(path).map(|f| f.data))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(SplashAssets::iter()
            .filter(|p| p.starts_with(path))
            .map(|p| SharedString::from(p.to_string()))
            .collect())
    }
}

/// Opens the window and holds it until `work` returns. `work` runs on a
/// background thread because GPUI wants the main one; it receives what the
/// player picks on the error screen.
///
/// Anything that has to happen after the download belongs inside `work`, not
/// after the call to `run_with`: `run` never comes back on macOS (`quit` goes
/// into `[NSApp terminate:]`) or on Windows (GPUI calls `ExitProcess`).
pub fn run_with<T, F>(rx: UnboundedReceiver<Update>, work: F) -> Option<T>
where
    T: Send + 'static,
    F: FnOnce(Choices) -> T + Send + 'static,
{
    let result: Arc<Mutex<Option<T>>> = Arc::new(Mutex::new(None));
    let slot = result.clone();
    let (choice_tx, choice_rx) = std::sync::mpsc::channel();

    gpui_platform::application()
        .with_assets(SplashAssetSource)
        .run(move |cx: &mut App| {
            for font in ["fonts/Monocraft-Bold.ttf", "fonts/Inter-Regular.ttf"] {
                if let Some(f) = SplashAssets::get(font) {
                    let _ = cx.text_system().add_fonts(vec![f.data]);
                }
            }

            let bounds = gpui::Bounds::centered(None, gpui::size(px(WINDOW.0), px(WINDOW.1)), cx);
            let _ = cx.open_window(
                WindowOptions {
                    window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some(SharedString::new_static("noro launcher")),
                        appears_transparent: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_window, cx| cx.new(|cx| Splash::new(rx, choice_tx, cx)),
            );

            // Off the main thread, which is busy drawing.
            let quit = cx.background_executor().spawn(async move {
                let value = work(choice_rx);
                slot.lock().replace(value);
            });
            cx.spawn(async move |cx| {
                quit.await;
                cx.update(|cx| cx.quit());
            })
            .detach();
        });

    Arc::try_unwrap(result).ok().and_then(|m| m.into_inner())
}
