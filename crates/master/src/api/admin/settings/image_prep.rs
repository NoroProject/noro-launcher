//! Подготовка залитых иллюстраций и признак прозрачности.

use crate::config::keys::{self, Key};
use crate::state::AppState;
use serde_json::Value;
use std::collections::BTreeMap;

/// Потолок для картинки, которую отдаём как есть. Анимацию не ужимаем, но и
/// стомегабайтную гифку на главной терпеть незачем — её грузил бы каждый гость.
const MAX_KEPT_IMAGE: usize = 8 * 1024 * 1024;

/// Подготовить картинку к раздаче.
///
/// GIF, WebP, PNG и APNG сохраняются байт в байт: перекодирование в JPEG,
/// которое здесь было раньше, убивало и анимацию, и прозрачность — залитая
/// зацикленная гифка превращалась в один кадр на белом фоне. Всё остальное
/// (прежде всего тяжёлые фотографии) по-прежнему ужимается.
pub fn prepare_image(data: &[u8]) -> Result<Vec<u8>, ImagePrepError> {
    if matches!(
        image::guess_format(data),
        Ok(image::ImageFormat::Gif | image::ImageFormat::WebP | image::ImageFormat::Png)
    ) {
        if data.len() > MAX_KEPT_IMAGE {
            return Err(ImagePrepError::TooLarge);
        }
        return Ok(data.to_vec());
    }

    let img = match image::load_from_memory(data) {
        Ok(i) => i,
        Err(_) if data.len() <= MAX_KEPT_IMAGE => return Ok(data.to_vec()),
        Err(e) => return Err(ImagePrepError::Decode(e)),
    };
    let img = if img.width() > 2048 || img.height() > 2048 {
        img.resize(2048, 2048, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let mut out = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut out);
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 92);
    img.write_with_encoder(encoder)
        .map_err(ImagePrepError::Decode)?;
    Ok(out)
}

/// Есть ли в картинке прозрачные места.
///
/// Смотрим на сами пиксели, а не на наличие альфа-канала: экспорт в RGBA сплошь
/// и рядом даёт полностью непрозрачную картинку с каналом, и по одному лишь
/// каналу сайт снимал бы подложку с обычного скриншота. Порог не 255, потому
/// что край вырезанного рендера почти всегда полупрозрачный от сглаживания.
pub fn has_transparency(data: &[u8]) -> bool {
    let Ok(img) = image::load_from_memory(data) else {
        // Формат, который мы не декодируем, отдаём как непрозрачный: подложка
        // лишней не выглядит, а вот снятая по ошибке — выглядит.
        return false;
    };
    if !img.color().has_alpha() {
        return false;
    }
    img.to_rgba8().pixels().any(|p| p[3] < 250)
}

/// Есть ли в картинке настройки `key` прозрачные места.
///
/// Признак пишет загрузчик, но у картинок, залитых до его появления, признака
/// нет — считаем один раз по самому файлу и запоминаем, иначе оператору
/// пришлось бы перезаливать всё уже стоящее.
///
/// Адрес, перебитый окружением, не трогаем: картинка там чужая, файла у нас
/// нет, и «непрозрачная» — единственный честный ответ.
pub async fn transparency(state: &AppState, stored: &BTreeMap<String, Value>, key: Key) -> bool {
    if crate::config::env_opt(key.env).is_some() {
        return false;
    }
    let flag = keys::transparency_key(key.name);
    if let Some(known) = stored.get(&flag).and_then(Value::as_bool) {
        return known;
    }

    let Some(sha1) = stored
        .get(key.name)
        .and_then(Value::as_str)
        .and_then(own_file_sha1)
        .filter(|sha1| state.files.exists(sha1))
    else {
        return false;
    };
    let Ok(data) = tokio::fs::read(state.files.path_for(&sha1)).await else {
        return false;
    };
    let Ok(transparent) = tokio::task::spawn_blocking(move || has_transparency(&data)).await else {
        return false;
    };

    // Запись не под правом оператора: считал её сервер, автора у неё нет.
    if let Err(e) = crate::db::set_setting(&state.db, &flag, &Value::Bool(transparent), None).await
    {
        tracing::warn!("не смог запомнить прозрачность {}: {e:#}", key.name);
    }
    transparent
}

/// sha1 из адреса, если он ведёт в наше файловое хранилище.
///
/// Единственное, что мы принимаем на вход, — имя файла: сорок hex-символов.
/// Всё остальное (чужой домен, путь с `..`) отсекается здесь, до обращения к
/// диску.
fn own_file_sha1(url: &str) -> Option<String> {
    let name = url.rsplit('/').next()?;
    let sha1 = name.split(['?', '#']).next()?;
    (sha1.len() == 40 && sha1.bytes().all(|b| b.is_ascii_hexdigit())).then(|| sha1.to_string())
}

pub enum ImagePrepError {
    TooLarge,
    Decode(image::ImageError),
}

impl std::fmt::Display for ImagePrepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImagePrepError::TooLarge => write!(f, "image is larger than 8 MB"),
            ImagePrepError::Decode(e) => write!(f, "{e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageFormat, RgbaImage};

    fn encode(img: DynamicImage, format: ImageFormat) -> Vec<u8> {
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), format)
            .unwrap();
        out
    }

    #[test]
    fn opaque_rgba_png_is_not_transparent() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, [10, 20, 30, 255].into()));
        assert!(!has_transparency(&encode(img, ImageFormat::Png)));
    }

    #[test]
    fn cut_out_render_is_transparent() {
        let mut raw = RgbaImage::from_pixel(4, 4, [10, 20, 30, 255].into());
        raw.put_pixel(0, 0, [0, 0, 0, 0].into());
        let img = DynamicImage::ImageRgba8(raw);
        assert!(has_transparency(&encode(img, ImageFormat::Png)));
    }

    #[test]
    fn jpeg_photo_is_not_transparent() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, [10, 20, 30, 255].into()));
        assert!(!has_transparency(&encode(img, ImageFormat::Jpeg)));
    }
}
