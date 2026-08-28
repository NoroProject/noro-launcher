//! Fonts and icons compiled into the binary.
//!
//! Has to be handed to `application().with_assets(AppAssets)`; without that GPUI
//! falls back to whatever the system has installed.

use gpui::SharedString;
use std::borrow::Cow;

#[derive(rust_embed::Embed)]
#[folder = "assets/"]
struct EmbeddedAssets;

pub struct AppAssets;

impl gpui::AssetSource for AppAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok(EmbeddedAssets::get(path).map(|f| f.data))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(EmbeddedAssets::iter()
            .filter(|p| p.starts_with(path))
            .map(|p| SharedString::from(p.to_string()))
            .collect())
    }
}

pub fn fonts() -> Vec<Cow<'static, [u8]>> {
    [
        "fonts/Inter-Regular.ttf",
        "fonts/PressStart2P-Regular.ttf",
        // Monocraft for small pixel type: the other open Minecraft-style faces
        // either have no Cyrillic or have gaps in it.
        "fonts/Monocraft.ttf",
        "fonts/Monocraft-Bold.ttf",
    ]
    .into_iter()
    .filter_map(|p| EmbeddedAssets::get(p).map(|f| f.data))
    .collect()
}
