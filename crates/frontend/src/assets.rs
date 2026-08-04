//! Источник встроенных ассетов (шрифты + иконки). Регистрация через
//! `application().with_assets(AppAssets)` — это и есть «фикс шрифтов»: GPUI
//! берёт шрифт Inter из бинарника, не завися от системных шрифтов.

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

/// Байты встроенных шрифтов для регистрации в текстовой системе GPUI.
pub fn fonts() -> Vec<Cow<'static, [u8]>> {
    [
        "fonts/Inter-Regular.ttf",
        "fonts/PressStart2P-Regular.ttf",
        // Шрифт мелкой типографики — открытая реализация майнкрафтовского.
        // Silkscreen отпал из-за отсутствия кириллицы, Pixelify Sans — из-за
        // дыр в ней (не было заглавных «О» и «П»), Handjet оказался слишком
        // узким. У Monocraft покрытие полное.
        "fonts/Monocraft.ttf",
        "fonts/Monocraft-Bold.ttf",
    ]
    .into_iter()
    .filter_map(|p| EmbeddedAssets::get(p).map(|f| f.data))
    .collect()
}
