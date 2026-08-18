//! Модель скина: классическая (Стив) или тонкая (Алекс).
//!
//! Моделей ровно две, и снаружи различие видно только по ширине рук: 4 пикселя
//! против 3. В самом PNG признака нет — есть договорённость, по которой у
//! тонкого скина недостающий столбец руки оставлен прозрачным. По ней модель и
//! угадывают все редакторы скинов, и по ней же угадываем мы.
//!
//! Угадывание — только умолчание. Последнее слово за игроком: скин может быть
//! нарисован под классику, но с прозрачным столбцом по недосмотру, и спорить с
//! человеком о его собственной картинке незачем.

/// `true` — похоже на тонкую модель.
///
/// Проверяем пиксель `(54, 20)`: у классического скина это край правой руки и
/// он непрозрачен, у тонкого — тот самый лишний столбец. Тот же признак читает
/// `skin_render_3d`, поэтому превью и игра сходятся.
///
/// Всё, что не разбирается как картинка 64×64 и шире, считается классическим:
/// у старых скинов 64×32 тонкой модели не бывает вовсе.
pub fn detect_slim(png: &[u8]) -> bool {
    let Ok(image) = image::load_from_memory(png) else {
        return false;
    };
    let rgba = image.to_rgba8();
    if rgba.width() != 64 || rgba.height() < 64 {
        return false;
    }
    rgba.get_pixel(54, 20)[3] == 0
}

/// Разбор того, что прислал клиент: `slim` / `classic`, либо `true` / `false`.
///
/// `None` — поля не было, и модель надо определить по картинке.
pub fn parse_choice(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "slim" | "alex" | "true" => Some(true),
        "classic" | "wide" | "steve" | "false" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_both_spellings() {
        assert_eq!(parse_choice("slim"), Some(true));
        assert_eq!(parse_choice(" CLASSIC "), Some(false));
        assert_eq!(parse_choice("true"), Some(true));
        assert_eq!(parse_choice("маловероятно"), None);
    }

    /// Мусор вместо картинки не должен ронять загрузку скина: файл всё равно
    /// не пройдёт проверку PNG-сигнатуры выше, а модель тут просто неизвестна.
    #[test]
    fn junk_is_classic() {
        assert!(!detect_slim(b"not a png at all"));
    }

    /// Прозрачный столбец руки — тонкая модель; залитый — классическая.
    #[test]
    fn transparent_arm_column_means_slim() {
        let mut wide = image::RgbaImage::from_pixel(64, 64, image::Rgba([1, 2, 3, 255]));
        let mut slim = wide.clone();
        slim.put_pixel(54, 20, image::Rgba([0, 0, 0, 0]));

        assert!(!detect_slim(&encode(&mut wide)));
        assert!(detect_slim(&encode(&mut slim)));
    }

    #[cfg(test)]
    fn encode(image: &mut image::RgbaImage) -> Vec<u8> {
        let mut out = std::io::Cursor::new(Vec::new());
        image
            .write_to(&mut out, image::ImageFormat::Png)
            .expect("png пишется");
        out.into_inner()
    }
}
