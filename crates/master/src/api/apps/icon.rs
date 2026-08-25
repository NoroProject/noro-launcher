//! Иконка приложения: одна и та же картинка у всех, кто её увидит.
//!
//! Приводим к общему виду сами, а не просим об этом автора. Иконка появляется
//! на экране согласия рядом с чужим логотипом и в списке доверенных приложений
//! кабинета — там разнобой размеров и пропорций читается как небрежность
//! инстанса, а не как небрежность автора.

/// Сторона иконки после обработки. 256 — вдвое больше самого крупного места,
/// где она показывается: хватает для экранов с двойной плотностью.
const SIDE: u32 = 256;

/// Что принимаем на вход. Меньше 64 пикселей — это не логотип, а фавикон:
/// растянутый до 256 он выглядит хуже, чем заглушка.
const MIN_SIDE: u32 = 64;
const MAX_UPLOAD: usize = 4 * 1024 * 1024;

#[derive(Debug)]
pub enum IconError {
    TooLarge,
    TooSmall(u32, u32),
    Decode(image::ImageError),
}

impl std::fmt::Display for IconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconError::TooLarge => write!(f, "the icon must be 4 MB or smaller"),
            IconError::TooSmall(w, h) => {
                write!(
                    f,
                    "the icon must be at least {MIN_SIDE}×{MIN_SIDE}, got {w}×{h}"
                )
            }
            IconError::Decode(e) => write!(f, "{e}"),
        }
    }
}

/// Привести иконку к квадрату 256×256 в PNG.
///
/// Обрезаем по центру, а не вписываем в поля: логотипы почти всегда квадратные
/// или близки к тому, а поля пришлось бы чем-то заливать — на тёмной панели
/// белая рамка вокруг чужого логотипа выглядит как ошибка загрузки.
///
/// PNG на выходе всегда: у логотипов прозрачный фон — правило, а не редкость,
/// и JPEG превратил бы его в чёрный прямоугольник.
pub fn prepare(data: &[u8]) -> Result<Vec<u8>, IconError> {
    if data.len() > MAX_UPLOAD {
        return Err(IconError::TooLarge);
    }
    let img = image::load_from_memory(data).map_err(IconError::Decode)?;
    if img.width() < MIN_SIDE || img.height() < MIN_SIDE {
        return Err(IconError::TooSmall(img.width(), img.height()));
    }

    let square = img.resize_to_fill(SIDE, SIDE, image::imageops::FilterType::Lanczos3);
    let mut out = Vec::new();
    square
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(IconError::Decode)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageFormat, RgbaImage};

    fn png(w: u32, h: u32) -> Vec<u8> {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, [1, 2, 3, 255].into()));
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), ImageFormat::Png)
            .unwrap();
        out
    }

    #[test]
    fn wide_icon_becomes_a_square() {
        let out = prepare(&png(512, 200)).unwrap();
        let img = image::load_from_memory(&out).unwrap();
        assert_eq!((img.width(), img.height()), (SIDE, SIDE));
    }

    #[test]
    fn tiny_icon_is_refused() {
        assert!(matches!(
            prepare(&png(32, 32)),
            Err(IconError::TooSmall(32, 32))
        ));
    }

    #[test]
    fn junk_is_not_an_icon() {
        assert!(matches!(
            prepare(b"not an image"),
            Err(IconError::Decode(_))
        ));
    }

    #[test]
    fn transparency_survives() {
        let mut raw = RgbaImage::from_pixel(128, 128, [9, 9, 9, 255].into());
        raw.put_pixel(0, 0, [0, 0, 0, 0].into());
        let mut src = Vec::new();
        DynamicImage::ImageRgba8(raw)
            .write_to(&mut std::io::Cursor::new(&mut src), ImageFormat::Png)
            .unwrap();

        let out = prepare(&src).unwrap();
        let img = image::load_from_memory(&out).unwrap();
        assert!(img.color().has_alpha());
        assert!(img.to_rgba8().pixels().any(|p| p[3] < 250));
    }
}
