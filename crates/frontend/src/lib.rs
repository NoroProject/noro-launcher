mod assets;
mod components;
mod console_controls;
mod console_model;
mod console_toolbar;
mod icons;
mod image_loader;
mod login;
mod pages;
mod perf;
mod skin;
mod skin_loader;
mod skin_preview;
mod state;
mod sync_text;
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

gpui::actions!(launcher, [CloseOverlay]);

const MAIN_WINDOW_SIZE: (f32, f32) = (1100., 720.);
const MAIN_WINDOW_MIN_SIZE: (f32, f32) = (1040., 680.);
/// Updates taken into the window in one go.
const MAX_UPDATE_BATCH: usize = 256;

impl gpui::Render for LauncherUI {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        // First thing in the frame: the counter has to see frames that draw
        // nothing as well.
        let sent = self.backend.sent_count();
        self.perf.frame(sent);

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
            | Page::Settings
            | Page::Account
            | Page::Messages => pages::launcher_shell(self, cx),
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
            .children(offline_banner(self))
            .child(div().flex_1().min_h_0().child(body))
            .when(!self.toasts.is_empty(), |d| {
                d.child(pages::toast_overlay(self, cx))
            })
            .children(pages::close_dialog::dialog(self, cx))
            .children(perf::overlay(self))
            // Esc closes the topmost panel or picker. Prompts that need an
            // answer (an admin's request, impersonation) are left to buttons:
            // dismissing those by a stray key would answer for the player.
            .on_action(cx.listener(|this, _: &CloseOverlay, _w, cx| {
                if this.close_top_overlay() {
                    cx.notify();
                }
            }))
    }
}

/// The master is out of reach. Without this an offline launcher looked like a
/// network with no servers, and nothing said it was trying to reconnect.
fn offline_banner(ui: &LauncherUI) -> Option<gpui::AnyElement> {
    if !ui.connection_lost {
        return None;
    }
    Some(
        div()
            .w_full()
            .flex_shrink_0()
            .px(px(16.))
            .py(px(6.))
            .bg(rgb(WARNING))
            .text_color(rgb(BG_WINDOW))
            .font_family(FONT_PIXEL_ALT)
            .text_size(px(12.))
            .child(i18n::t("offline-banner"))
            .into_any_element(),
    )
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
        move |window, cx| {
            let handle = window.window_handle();
            let view = cx.new(|_cx| {
                let mut ui = LauncherUI::new(backend_handle.clone());
                ui.main_window = Some(handle);
                ui
            });
            let view_weak: WeakEntity<LauncherUI> = view.downgrade();
            cx.set_global(GlobalLauncherUI(view.clone()));

            // The skin preview animates only while the window has the focus;
            // coming back to it picks the animation up again.
            view.update(cx, |_, cx| {
                cx.observe_window_activation(window, |ui, window, cx| {
                    ui.window_active = window.is_window_active();
                    if ui.window_active && ui.page == Page::Profile {
                        ui.start_skin_animation(cx);
                    }
                })
                .detach();
            });

            // Alt+F4 and the system's own close go through the same question
            // as the close button.
            let closing = view.downgrade();
            window.on_window_should_close(cx, move |_window, cx| {
                let allowed = closing
                    .update(cx, |ui, cx| {
                        let allowed = ui.request_close();
                        cx.notify();
                        allowed
                    })
                    .unwrap_or(true);
                if allowed {
                    cx.defer(|cx| cx.quit());
                }
                allowed
            });

            cx.spawn({
                let frontend_recv = frontend_recv.clone();
                move |async_app: &mut AsyncApp| {
                    let mut async_app = async_app.clone();
                    async move {
                        let mut recv = frontend_recv.lock().await;
                        #[cfg(debug_assertions)]
                        let mut lagging = false;
                        loop {
                            let Some(first) = recv.recv().await else {
                                break;
                            };
                            // Whatever is already queued goes in the same
                            // update: one trip into the app per message made a
                            // burst slower to take in than to send.
                            let mut batch = vec![first];
                            while batch.len() < MAX_UPDATE_BATCH {
                                let Some(msg) = recv.try_recv() else { break };
                                batch.push(msg);
                            }
                            // The channel no longer drops anything, so falling
                            // behind shows up as a delay; say so while developing.
                            #[cfg(debug_assertions)]
                            {
                                let behind = recv.backlog() > MAX_UPDATE_BATCH;
                                if behind && !lagging {
                                    tracing::warn!(
                                        backlog = recv.backlog(),
                                        "the window is falling behind the backend"
                                    );
                                }
                                lagging = behind;
                            }
                            let updated = view_weak.update(&mut async_app, |state, cx| {
                                for msg in batch {
                                    state.on_message(msg, cx);
                                }
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
            cx.bind_keys([gpui::KeyBinding::new("escape", CloseOverlay, None)]);
            open_window(cx, backend_handle.clone(), frontend_recv.clone());
            cx.activate(true);
        });
}
