//! The screenshots tab: the list for the server it shows, and the thumbnails
//! decoded for it.
//!
//! Thumbnails are GPU textures, and GPUI never evicts one on its own. The tab
//! keeps only one server's worth and hands them back when it moves to another
//! server or a file disappears from the list.

use super::*;
use bridge::ScreenshotInfo;
use std::path::PathBuf;

/// Thumbnails per screen before "show more". A folder of a thousand
/// screenshots decoded at once would be a gigabyte of textures.
pub const GALLERY_PAGE: usize = 24;
/// Longest side of a thumbnail, at twice the card size for sharp HiDPI.
const THUMB_SIDE: u32 = 480;

#[derive(Default)]
pub struct Gallery {
    pub server_id: Option<Uuid>,
    pub items: Vec<ScreenshotInfo>,
    /// The list has arrived for `server_id` at least once: before that the tab
    /// says "loading", not "no screenshots".
    pub loaded: bool,
    pub shown: usize,
    pub thumbs: HashMap<PathBuf, Arc<RenderImage>>,
    loading: HashSet<PathBuf>,
    failed: HashSet<PathBuf>,
}

impl LauncherUI {
    pub fn open_screenshots(&mut self, server_id: Uuid, cx: &mut Context<Self>) {
        if self.gallery.server_id != Some(server_id) {
            self.drop_gallery_thumbs(|_| true, cx);
            self.gallery = Gallery {
                server_id: Some(server_id),
                ..Gallery::default()
            };
        }
        self.gallery.shown = GALLERY_PAGE;
        self.page = Page::ServerScreenshots(server_id);
        self.backend
            .send(MessageToBackend::RequestScreenshots { server_id });
    }

    pub fn refresh_screenshots(&mut self, server_id: Uuid) {
        self.backend
            .send(MessageToBackend::RequestScreenshots { server_id });
    }

    pub(super) fn on_screenshots(
        &mut self,
        server_id: Uuid,
        items: Vec<ScreenshotInfo>,
        cx: &mut Context<Self>,
    ) {
        if self.gallery.server_id != Some(server_id) {
            return;
        }
        let present: HashSet<PathBuf> = items.iter().map(|i| i.path.clone()).collect();
        self.drop_gallery_thumbs(|path| !present.contains(path), cx);
        self.gallery.failed.retain(|p| present.contains(p));
        self.gallery.items = items;
        self.gallery.loaded = true;
    }

    /// A finished game may have left new screenshots behind.
    pub(super) fn refresh_gallery_after_game(&mut self, server_id: Uuid) {
        if self.page == Page::ServerScreenshots(server_id) {
            self.refresh_screenshots(server_id);
        }
    }

    pub fn show_more_screenshots(&mut self) {
        self.gallery.shown = (self.gallery.shown + GALLERY_PAGE).min(self.gallery.items.len());
    }

    pub fn open_screenshot(&self, server_id: Uuid, file_name: String) {
        self.backend.send(MessageToBackend::OpenScreenshot {
            server_id,
            file_name,
        });
    }

    pub fn delete_screenshot(&self, server_id: Uuid, file_name: String) {
        self.backend.send(MessageToBackend::DeleteScreenshot {
            server_id,
            file_name,
        });
    }

    pub fn open_screenshots_folder(&self, server_id: Uuid) {
        self.backend
            .send(MessageToBackend::OpenScreenshotsFolder { server_id });
    }

    pub fn ensure_screenshot_thumb(&mut self, path: &PathBuf, cx: &mut Context<Self>) {
        let g = &mut self.gallery;
        if g.thumbs.contains_key(path) || g.loading.contains(path) || g.failed.contains(path) {
            return;
        }
        g.loading.insert(path.clone());
        let path = path.clone();
        cx.spawn(async move |this, cx| {
            let result =
                crate::image_loader::load_render_image_file(path.clone(), THUMB_SIDE).await;
            let _ = this.update(cx, |ui, cx| {
                // Gone from the list meanwhile (another server, a delete): the
                // texture was never drawn, so there is nothing to give back.
                if !ui.gallery.loading.remove(&path) {
                    return;
                }
                match result {
                    Ok(image) => {
                        ui.gallery.thumbs.insert(path, image);
                    }
                    Err(_) => {
                        ui.gallery.failed.insert(path);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn drop_gallery_thumbs(&mut self, drop: impl Fn(&PathBuf) -> bool, cx: &mut Context<Self>) {
        let gone: Vec<PathBuf> = self
            .gallery
            .thumbs
            .keys()
            .filter(|p| drop(p))
            .cloned()
            .collect();
        for path in gone {
            if let Some(image) = self.gallery.thumbs.remove(&path) {
                if Arc::strong_count(&image) == 1 {
                    cx.drop_image(image, None);
                }
            }
        }
        self.gallery.loading.retain(|p| !drop(p));
    }
}
