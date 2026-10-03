//! Pictures from the network: server backgrounds and icons, catalogue icons,
//! screenshots, skin preset renders and news images.

use super::*;

impl LauncherUI {
    pub fn ensure_background_loaded(
        &mut self,
        server_id: Uuid,
        url: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(url) = url.filter(|u| !u.trim().is_empty()) else {
            return;
        };
        if self.image_failed.contains(&url)
            || (self.background_image_urls.get(&server_id) == Some(&url)
                && (self.background_images.contains_key(&server_id)
                    || self.background_loading.contains(&server_id)))
        {
            return;
        }

        // The old picture stays up until the new one is in; replacing it is
        // also what hands its texture back to GPUI below. Removing it first
        // meant the replacement never found anything to release.
        self.background_image_urls.insert(server_id, url.clone());
        self.background_loading.insert(server_id);
        cx.spawn(async move |this, cx| {
            let expected_url = url.clone();
            let result = crate::image_loader::load_render_image_capped(url, 1600).await;
            let _ = this.update(cx, |state, cx| {
                state.background_loading.remove(&server_id);
                if state.background_image_urls.get(&server_id) != Some(&expected_url) {
                    return;
                }
                match result {
                    Ok(image) => {
                        // Hand the old texture back to GPUI: the atlas keeps every
                        // `RenderImage` by id and never evicts one on its own, so
                        // changing the background would leave megabytes behind.
                        if let Some(stale) = state.background_images.insert(server_id, image) {
                            if Arc::strong_count(&stale) == 1 {
                                cx.drop_image(stale, None);
                            }
                        }
                    }
                    Err(err) => {
                        // Once per URL. A failure used not to be remembered, and
                        // the next frame downloaded again, with a new error toast
                        // for every attempt.
                        state.image_failed.insert(expected_url);
                        let mut args = i18n::FluentArgs::new();
                        args.set("reason", err.to_string());
                        state.notify_toast(
                            i18n::t_args("error-background-failed", &args),
                            NotifLevel::Warning,
                            cx,
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn ensure_icon_loaded(
        &mut self,
        server_id: Uuid,
        url: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(url) = url.filter(|u| !u.trim().is_empty()) else {
            return;
        };
        if self.image_failed.contains(&url)
            || (self.server_icon_urls.get(&server_id) == Some(&url)
                && (self.server_icons.contains_key(&server_id)
                    || self.icons_loading.contains(&server_id)))
        {
            return;
        }
        self.server_icon_urls.insert(server_id, url.clone());
        self.icons_loading.insert(server_id);
        cx.spawn(async move |this, cx| {
            let expected_url = url.clone();
            let result = crate::image_loader::load_render_image_capped(url, 256).await;
            let _ = this.update(cx, |state, cx| {
                state.icons_loading.remove(&server_id);
                if state.server_icon_urls.get(&server_id) != Some(&expected_url) {
                    return;
                }
                match result {
                    Ok(image) => {
                        if let Some(stale) = state.server_icons.insert(server_id, image) {
                            if Arc::strong_count(&stale) == 1 {
                                cx.drop_image(stale, None);
                            }
                        }
                    }
                    Err(_) => {
                        state.image_failed.insert(expected_url);
                    }
                };
                cx.notify();
            });
        })
        .detach();
    }

    /// How many images never loaded. For the overlay, to tell "no icons"
    /// from "icons aren't arriving".
    pub fn failed_image_count(&self) -> usize {
        self.optional_mod_icons_failed.len() + self.image_failed.len()
    }

    pub fn ensure_optional_mod_icon_loaded(&mut self, url: Option<String>, cx: &mut Context<Self>) {
        self.ensure_remote_image_loaded(url, ICON_SIDE, cx);
    }

    /// Screenshots are shown much larger than icons; decoded at icon size they
    /// came out blurry.
    pub fn ensure_screenshot_loaded(&mut self, url: Option<String>, cx: &mut Context<Self>) {
        self.ensure_remote_image_loaded(url, SCREENSHOT_SIDE, cx);
    }

    pub(super) fn ensure_remote_image_loaded(
        &mut self,
        url: Option<String>,
        max_side: u32,
        cx: &mut Context<Self>,
    ) {
        let Some(url) = url.filter(|u| !u.trim().is_empty()) else {
            return;
        };
        if self.optional_mod_icons.contains_key(&url)
            || self.optional_mod_icons_loading.contains(&url)
            || self.optional_mod_icons_failed.contains(&url)
        {
            return;
        }
        self.optional_mod_icons_loading.insert(url.clone());
        cx.spawn(async move |this, cx| {
            let result = crate::image_loader::load_render_image_capped(url.clone(), max_side).await;
            let _ = this.update(cx, |state, cx| {
                state.optional_mod_icons_loading.remove(&url);
                match result {
                    Ok(image) => {
                        // Catalogue pages, avatars and screenshots all land
                        // here, and nothing ever left. Past the cap the lot is
                        // released; what is still on screen loads again.
                        if state.optional_mod_icons.len() >= REMOTE_IMAGE_CAP {
                            for (_, stale) in state.optional_mod_icons.drain() {
                                cx.drop_image(stale, None);
                            }
                        }
                        state.optional_mod_icons.insert(url, image);
                        // Redraw only when there is something to show: otherwise
                        // a failure triggers the very frame that repeats it.
                        cx.notify();
                    }
                    Err(e) => {
                        tracing::debug!(url = %url, error = %e, "icon did not load");
                        state.optional_mod_icons_failed.insert(url);
                    }
                }
            });
        })
        .detach();
    }

    /// Every removal hands the texture back to GPUI, whose atlas never evicts
    /// anything on its own.
    pub(super) fn clear_background(&mut self, server_id: Uuid, cx: &mut Context<Self>) {
        if let Some(image) = self.background_images.remove(&server_id) {
            cx.drop_image(image, None);
        }
        self.background_loading.remove(&server_id);
        self.background_image_urls.remove(&server_id);
    }

    pub(super) fn clear_icon(&mut self, server_id: Uuid, cx: &mut Context<Self>) {
        if let Some(image) = self.server_icons.remove(&server_id) {
            cx.drop_image(image, None);
        }
        self.icons_loading.remove(&server_id);
        self.server_icon_urls.remove(&server_id);
    }

    pub fn load_preset_renders(&mut self, cx: &mut Context<Self>) {
        if self.preset_images.contains_key("steve") {
            return;
        }
        let master_url = self.config.master_url.clone();
        let presets = [
            "steve", "alex", "ari", "zuri", "efe", "makena", "kai", "sunny", "noor",
        ];
        for preset in presets {
            let name = preset.to_string();
            let url = format!(
                "{}/api/textures/renders/bust?preset={}&scale=8&yaw=-25&pitch=12",
                master_url.trim_end_matches('/'),
                name
            );
            cx.spawn(async move |this, cx| {
                if let Ok(img) =
                    crate::image_loader::load_image_capped(url, PRESET_RENDER_SIDE).await
                {
                    let _ = this.update(cx, |this, cx| {
                        this.preset_images.insert(name, img);
                        cx.notify();
                    });
                }
            })
            .detach();
        }
    }

    /// News images load lazily: the list doesn't need them, and there can be a
    /// lot of news.
    pub fn load_news_image(&mut self, id: Uuid, cx: &mut Context<Self>) {
        if self.news_images.contains_key(&id) || self.news_images_loading.contains(&id) {
            return;
        }
        let Some(url) = self
            .news
            .iter()
            .find(|n| n.id == id)
            .and_then(|n| n.preview_img_url.clone())
            .filter(|u| !u.trim().is_empty())
        else {
            return;
        };

        self.news_images_loading.insert(id);
        cx.spawn(async move |this, cx| {
            let result = crate::image_loader::load_image_capped(url, 1200).await;
            let _ = this.update(cx, |state, cx| {
                state.news_images_loading.remove(&id);
                if let Ok(image) = result {
                    state.news_images.insert(id, image);
                }
                cx.notify();
            });
        })
        .detach();
    }
}
