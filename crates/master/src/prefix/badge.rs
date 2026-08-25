//! Плашка роли картинкой: градиентный фон и надпись поверх.

use super::font;
use image::{ImageEncoder, Rgba, RgbaImage};

/// Поле слева и справа. Ровно пиксель — как в эталоне: «ADMIN» там укладывается
/// в 25 пикселей, где 23 занимают буквы, и по одному остаётся с краёв.
const PAD: usize = 1;

/// Поле сверху и снизу: пять строк букв плюс два поля дают ровно семь пикселей
/// высоты, как в эталоне. Без полей буквы упираются в край, и плашка выглядит
/// обрезанной, а не свёрстанной.
const PAD_Y: usize = 1;

/// Во сколько раз тускнеет дальний край градиента.
///
/// Было 0.55, и правый край уходил в темноту так, что последние буквы в нём
/// тонули. Градиент должен быть виден, а не мешать читать.
const FADE: f32 = 0.72;

/// Тень под буквой — как в эталоне: почти прозрачная, она не читается сама, но
/// не даёт светлой надписи слиться со светлым концом градиента.
const SHADOW: Rgba<u8> = Rgba([0x22, 0x22, 0x22, 0x44]);

/// Цвет `#rrggbb` в тройку. Мусор в поле роли не должен ронять сборку пака.
pub fn rgb(hex: &str) -> (u8, u8, u8) {
    let clean = hex.trim().trim_start_matches('#');
    let parse = |at: usize| u8::from_str_radix(clean.get(at..at + 2).unwrap_or("80"), 16);
    match (parse(0), parse(2), parse(4)) {
        (Ok(r), Ok(g), Ok(b)) if clean.len() >= 6 => (r, g, b),
        _ => (0x9C, 0xA3, 0xAF),
    }
}

/// Насколько цвет светлый — по нему выбирается цвет надписи.
///
/// Порог по воспринимаемой яркости, а не по сумме каналов: глаз видит зелёный
/// куда сильнее синего. Порог высокий намеренно: белая надпись с тёмной тенью
/// читается почти на любом фоне, и переключаться на тёмную стоит только там, где
/// белого уже не видно, — на жёлтом и салатовом.
fn light(color: (u8, u8, u8)) -> bool {
    let (r, g, b) = color;
    (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) > 200.0
}

/// Высота плашки: глиф плюс поля. По ней же считается подъём над базовой
/// линией в описании шрифта.
pub fn height() -> usize {
    font::HEIGHT + 2 * PAD_Y
}

/// Плашка в PNG. Ширина — по тексту, высота постоянная.
pub fn png(text: &str, color: &str) -> anyhow::Result<Vec<u8>> {
    let base = rgb(color);
    let text = text.trim();
    let width = font::width(text) + 2 * PAD;
    let height = font::HEIGHT + 2 * PAD_Y;
    let mut img = RgbaImage::new(width as u32, height as u32);

    gradient(&mut img, base);

    // Надпись рисуется дважды: сначала тень, потом сама буква поверх неё. Без
    // тени текст растворяется в том конце градиента, что ближе к его цвету.
    let dark_text = light(base);
    let ink = if dark_text {
        Rgba([0x16, 0x16, 0x16, 0xFF])
    } else {
        Rgba([0xFF, 0xFF, 0xFF, 0xFF])
    };
    // Сдвиг только вправо, не по диагонали: у эталона строки над буквами и под
    // ними чистые, а тень от нижнего ряда легла бы как раз в нижнюю.
    font::dots(text, |x, y| put(&mut img, x + PAD + 1, y + PAD_Y, SHADOW));
    font::dots(text, |x, y| put(&mut img, x + PAD, y + PAD_Y, ink));

    let mut out = Vec::new();
    image::codecs::png::PngEncoder::new(&mut out).write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgba8,
    )?;
    Ok(out)
}

/// Градиент вбок: от цвета роли к его же затемнённому краю.
fn gradient(img: &mut RgbaImage, base: (u8, u8, u8)) {
    let width = img.width().max(1) as f32;
    for x in 0..img.width() {
        let t = x as f32 / width;
        let fade = 1.0 - t * (1.0 - FADE);
        let shade = Rgba([
            (base.0 as f32 * fade) as u8,
            (base.1 as f32 * fade) as u8,
            (base.2 as f32 * fade) as u8,
            0xFF,
        ]);
        for y in 0..img.height() {
            img.put_pixel(x, y, shade);
        }
    }
}

/// Кладёт пиксель со смешиванием по альфе. За краем — молча мимо: тень
/// последнего столбца выходит за плашку, и это нормально.
fn put(img: &mut RgbaImage, x: usize, y: usize, color: Rgba<u8>) {
    if x >= img.width() as usize || y >= img.height() as usize {
        return;
    }
    let under = *img.get_pixel(x as u32, y as u32);
    let a = color.0[3] as f32 / 255.0;
    let mix = |over: u8, under: u8| (over as f32 * a + under as f32 * (1.0 - a)) as u8;
    img.put_pixel(
        x as u32,
        y as u32,
        Rgba([
            mix(color.0[0], under.0[0]),
            mix(color.0[1], under.0[1]),
            mix(color.0[2], under.0[2]),
            0xFF,
        ]),
    );
}

/// Плашка вместе с ником — как строка чата выглядит целиком.
///
/// Нужна предпросмотру: рисовать ник шрифтом сайта рядом с пиксельной плашкой
/// бессмысленно — они из разных миров, и по такой картинке нельзя понять, как
/// оно сядет в игре. Ник рисуется тем же шрифтом 5×7, что и сама плашка.
pub fn line(text: &str, color: &str, name: &str) -> anyhow::Result<Vec<u8>> {
    let badge = png(text, color)?;
    let plate = image::load_from_memory(&badge)?.to_rgba8();

    // Пробел между плашкой и ником — тот же, что агент ставит в чате.
    let gap = 4;
    let width = plate.width() + gap + font::width(name) as u32;
    let mut img = RgbaImage::new(width, plate.height());

    for (x, y, pixel) in plate.enumerate_pixels() {
        img.put_pixel(x, y, *pixel);
    }
    let left = (plate.width() + gap) as usize;
    let ink = Rgba([0xFF, 0xFF, 0xFF, 0xFF]);
    font::dots(name, |x, y| put(&mut img, left + x, y + PAD_Y, ink));

    let mut out = Vec::new();
    image::codecs::png::PngEncoder::new(&mut out).write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgba8,
    )?;
    Ok(out)
}
