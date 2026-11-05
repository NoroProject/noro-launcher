//! The instance's name and logo in the window: title bar, sidebar, login.
//!
//! The backend reads them from the master's public settings (see
//! `backend::branding`); until then the name is the one the bootstrapper was
//! stamped with, and the logo is the one the launcher ships with.

use super::*;
use gpui::SharedString;

/// The sidebar draws the logo at 26 px; twice that keeps it sharp on HiDPI.
const LOGO_SIDE: u32 = 64;

pub struct Brand {
    pub name: SharedString,
    /// `None` until the operator's logo is downloaded, and for good when there
    /// is none.
    pub logo: Option<Arc<RenderImage>>,
    logo_url: Option<String>,
}

impl Default for Brand {
    fn default() -> Self {
        Self {
            name: bridge::brand::initial_instance_name().into(),
            logo: None,
            logo_url: None,
        }
    }
}

impl LauncherUI {
    pub(super) fn on_branding(
        &mut self,
        name: String,
        logo_url: Option<String>,
        cx: &mut Context<Self>,
    ) {
        if self.brand.name != name {
            self.brand.name = name.clone().into();
            // Deferred: this runs inside an update of the window's own view.
            if let Some(handle) = self.main_window {
                cx.defer(move |cx| {
                    let _ = handle.update(cx, |_, window, _| window.set_window_title(&name));
                });
            }
        }
        if self.brand.logo_url == logo_url {
            return;
        }
        self.brand.logo_url = logo_url.clone();
        if let Some(stale) = self.brand.logo.take() {
            cx.drop_image(stale, None);
        }
        let Some(url) = logo_url else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let result =
                crate::image_loader::load_render_image_capped(url.clone(), LOGO_SIDE).await;
            let _ = this.update(cx, |state, cx| {
                // A newer logo may have been set while this one downloaded.
                if state.brand.logo_url.as_deref() != Some(url.as_str()) {
                    return;
                }
                match result {
                    Ok(image) => {
                        state.brand.logo = Some(image);
                        cx.notify();
                    }
                    Err(e) => tracing::debug!(url = %url, error = %e, "instance logo did not load"),
                }
            });
        })
        .detach();
    }
}
