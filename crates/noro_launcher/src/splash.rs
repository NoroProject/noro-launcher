//! Окно загрузки: логотип, маскот и полоса прогресса, пока качается core.
//!
//! Раньше на этом месте не было ничего — игрок жал ярлык и минуту смотрел в
//! пустой рабочий стол. Рисуем тем же GPUI, что и лаунчер, чтобы экран
//! совпадал с остальным интерфейсом, а не выглядел чужой заплаткой.

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

/// Что показывать в окне. Присылается из потока закачки.
#[derive(Default, Clone)]
pub struct Progress {
    pub label: String,
    pub done: u64,
    pub total: u64,
}

/// Куда закачка шлёт свои отчёты.
pub type Reporter = UnboundedSender<Progress>;

struct Splash {
    progress: Progress,
}

impl Splash {
    fn new(mut rx: UnboundedReceiver<Progress>, cx: &mut Context<Self>) -> Self {
        // Перерисовываем по событию, а не по таймеру: задача спит на recv, пока
        // закачка не пришлёт новую цифру, и лишних кадров не возникает.
        cx.spawn(async move |this, cx| {
            while let Some(next) = rx.recv().await {
                if this
                    .update(cx, |splash, cx| {
                        splash.progress = next;
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
        }
    }
}

impl Render for Splash {
    fn render(&mut self, _w: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let p = self.progress.clone();
        let ratio = if p.total > 0 {
            (p.done as f32 / p.total as f32).clamp(0., 1.)
        } else {
            0.
        };

        div()
            .size_full()
            .bg(rgb(BG))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(16.))
            .child(img("mascot-loading.png").h(px(112.)).w(px(95.)))
            .child(
                div()
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
                    ),
            )
            .child(bar(ratio))
            .child(
                div()
                    .font_family("Monocraft")
                    .text_size(px(13.))
                    .text_color(rgb(MUTED))
                    .child(SharedString::from(p.label)),
            )
    }
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

/// Что позвать, когда работа закончилась, но окно ещё живо.
pub type OnDone<T> = Box<dyn FnOnce(&T) + Send + 'static>;

/// Показать окно и держать его, пока `work` не закончит. Возвращает результат
/// работы: сама закачка идёт в фоне, GPUI требует главный поток себе.
///
/// `on_done` получает результат сразу, как только `work` его вернул, — до
/// `cx.quit()`. Это единственная точка, которая срабатывает на всех платформах:
/// `run` не возвращает управление ни на macOS (`quit` упирается в
/// `[NSApp terminate:]`), ни на Windows (GPUI зовёт `ExitProcess`). Всё, что
/// должно случиться после закачки, вешается сюда, а не на код после `run_with`.
pub fn run_with<T, F>(
    rx: UnboundedReceiver<Progress>,
    work: F,
    on_done: Option<OnDone<T>>,
) -> Option<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let result: Arc<Mutex<Option<T>>> = Arc::new(Mutex::new(None));
    let slot = result.clone();

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
                |_window, cx| cx.new(|cx| Splash::new(rx, cx)),
            );

            // Работа идёт в отдельном потоке: главный занят отрисовкой.
            let quit = cx.background_executor().spawn(async move {
                let value = work();
                if let Some(cb) = on_done {
                    cb(&value);
                }
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
