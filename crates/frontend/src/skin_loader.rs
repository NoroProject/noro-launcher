//! Loading the profile skin, cape and avatar textures.

use crate::state::LauncherUI;
use gpui::Context;

impl LauncherUI {
    pub fn load_user_skin(&mut self, cx: &mut Context<Self>) {
        let Some(user) = self.user.clone() else {
            return;
        };

        // Забираем до того, как поля профиля разъедутся по замыканиям ниже.
        let avatar_url = user.avatar_url().map(str::to_string);

        // Invalidate on texture change (new url after an upload).
        if self.skin_url != user.skin_url {
            self.skin_url = user.skin_url.clone();
            self.skin_uploading = false;
            self.reset_skin_preview();
        }
        if self.cape_url != user.cape_url {
            self.cape_url = user.cape_url.clone();
            self.cape_bytes = None;
        }

        if let Some(url) = user
            .skin_url
            .filter(|_| !self.skin_loading && self.skin_bytes.is_none())
        {
            self.skin_loading = true;
            cx.spawn(async move |this, cx| {
                let loaded = crate::image_loader::load_image_and_bytes(url).await;
                let _ = this.update(cx, |this, cx| {
                    if let Ok((img, bytes)) = loaded {
                        this.skin_image = Some(img);
                        this.skin_bytes = Some(bytes);
                    }
                    this.skin_loading = false;
                    this.refresh_skin_preview(cx);
                    cx.notify();
                });
            })
            .detach();
        }

        if let Some(url) = user
            .cape_url
            .filter(|_| !self.cape_loading && self.cape_bytes.is_none())
        {
            self.cape_loading = true;
            cx.spawn(async move |this, cx| {
                let loaded = crate::image_loader::load_image_and_bytes(url).await;
                let _ = this.update(cx, |this, cx| {
                    if let Ok((_, bytes)) = loaded {
                        this.cape_bytes = Some(bytes);
                    }
                    this.cape_loading = false;
                    this.refresh_skin_preview(cx);
                    cx.notify();
                });
            })
            .detach();
        }

        self.load_avatar(avatar_url, cx);
    }

    fn load_avatar(&mut self, avatar_url: Option<String>, cx: &mut Context<Self>) {
        let Some(url) = avatar_url.filter(|_| !self.avatar_loading) else {
            return;
        };
        self.avatar_loading = true;
        cx.spawn(async move |this, cx| {
            let loaded = crate::image_loader::load_image_from_url(url).await;
            let _ = this.update(cx, |this, cx| {
                if let Ok(img) = loaded {
                    this.avatar_image = Some(img);
                }
                this.avatar_loading = false;
                cx.notify();
            });
        })
        .detach();
    }

    pub(crate) fn reset_skin_preview(&mut self) {
        self.skin_image = None;
        self.skin_preview = None;
        self.skin_bytes = None;
        self.skin_yaw = 0.0;
        self.skin_sway = 0.0;
    }

    /// Запустить цикл отрисовки. Ждём плащ, иначе первые кадры уйдут без него.
    fn refresh_skin_preview(&mut self, cx: &mut Context<Self>) {
        if self.cape_loading {
            return;
        }
        self.start_skin_animation(cx);
    }
}
