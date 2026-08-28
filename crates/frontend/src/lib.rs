mod assets;
mod components;
mod console_controls;
mod console_model;
mod console_toolbar;
mod icons;
mod image_loader;
mod login;
mod pages;
mod skin;
mod skin_loader;
mod skin_preview;
mod state;
mod theme;

use bridge::{BackendHandle, FrontendReceiver};
use gpui::{
    div, point, prelude::*, px, rgb, App, AsyncApp, IntoElement, SharedString, WeakEntity, Window,
    WindowOptions,
};
use gpui_platform::application;
use std::sync::Arc;

pub use state::{GlobalLauncherUI, LauncherUI, Page};
use theme::*;

const MAIN_WINDOW_SIZE: (f32, f32) = (1100., 720.);
const MAIN_WINDOW_MIN_SIZE: (f32, f32) = (1040., 680.);

impl gpui::Render for LauncherUI {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let body = match self.page.clone() {
            Page::Login => login::render(self, cx),
            Page::Servers
            | Page::ServerDetail(_)
            | Page::ServerMods(_)
            | Page::ServerModCatalog(_)
            | Page::ServerSettings(_)
            | Page::News
            | Page::NewsDetail(_)
            | Page::Profile
            | Page::Settings => pages::launcher_shell(self, cx),
        };

        let compact_chrome = self.page != Page::Login;
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(BG_WINDOW))
            .font_family(FONT_PIXEL_ALT)
            .text_color(rgb(TEXT_PRIMARY))
            .text_sm()
            .child(components::window_chrome(compact_chrome, self, cx))
            .child(div().flex_1().min_h_0().child(body))
            .when(self.toast.is_some(), |d| {
                d.child(pages::toast_overlay(self, cx))
            })
    }
}

fn open_window(
    cx: &mut App,
    backend_handle: BackendHandle,
    frontend_recv: Arc<tokio::sync::Mutex<FrontendReceiver>>,
) {
    let bounds = gpui::Bounds::centered(
        None,
        gpui::size(px(MAIN_WINDOW_SIZE.0), px(MAIN_WINDOW_SIZE.1)),
        cx,
    );
    let _ = cx.open_window(
        WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            window_min_size: Some(gpui::size(
                px(MAIN_WINDOW_MIN_SIZE.0),
                px(MAIN_WINDOW_MIN_SIZE.1),
            )),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some(SharedString::new_static("noro launcher")),
                // Transparent titlebar, traffic lights parked off-screen: the
                // chrome is drawn by `components::window_chrome`.
                appears_transparent: true,
                traffic_light_position: Some(point(px(-120.), px(-120.))),
            }),
            ..Default::default()
        },
        move |_window, cx| {
            let view = cx.new(|_cx| LauncherUI::new(backend_handle.clone()));
            let view_weak: WeakEntity<LauncherUI> = view.downgrade();
            cx.set_global(GlobalLauncherUI(view.clone()));

            cx.spawn({
                let frontend_recv = frontend_recv.clone();
                move |async_app: &mut AsyncApp| {
                    let mut async_app = async_app.clone();
                    async move {
                        let mut recv = frontend_recv.lock().await;
                        loop {
                            let Some(msg) = recv.recv().await else { break };
                            let updated = view_weak.update(&mut async_app, |state, cx| {
                                state.on_message(msg, cx);
                            });
                            if updated.is_err() {
                                break;
                            }
                        }
                    }
                }
            })
            .detach();

            view
        },
    );
}

/// Blocks the calling thread until the app exits.
pub fn start(backend_handle: BackendHandle, frontend_recv: FrontendReceiver) {
    let frontend_recv = Arc::new(tokio::sync::Mutex::new(frontend_recv));

    application()
        .with_assets(assets::AppAssets)
        .run(move |cx: &mut App| {
            let _ = cx.text_system().add_fonts(assets::fonts());
            open_window(cx, backend_handle.clone(), frontend_recv.clone());
            cx.activate(true);
        });
}
