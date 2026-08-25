//! Сборка ресурспака с плашками: по PNG на роль плюс описание шрифта.

use super::{badge, codepoint};
use anyhow::Context;
use serde::Serialize;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Роль, какой она нужна паку: чем подписать плашку и каким цветом залить.
pub struct Role {
    /// Идентификатор роли — им называется файл текстуры.
    pub name: String,
    /// Что написать на плашке: поле `prefix`, а если пусто — `display_name`.
    pub text: String,
    /// Цвет роли, `#rrggbb`.
    pub color: String,
    /// Своя картинка вместо нарисованной. Ни градиент, ни текст к ней уже не
    /// подмешиваются: раз её загрузили руками, значит она и есть плашка.
    pub image: Option<Vec<u8>>,
}

/// Готовый пак и таблица «роль → символ», по которой агент собирает префикс.
pub struct Built {
    pub zip: Vec<u8>,
    pub glyphs: Vec<(String, char)>,
}

/// Диапазон версий пака.
///
/// Формат менялся почти каждое обновление, но шрифтов это не касалось: описание
/// провайдеров не двигалось с 1.13. Верхняя граница взята с запасом, чтобы пак
/// не начинал ругаться на каждой новой версии игры.
/// Основной формат. Клиент смотрит на него первым, и версия должна быть той,
/// на которой сервер и работает, — иначе пак помечается несовместимым ещё до
/// того, как будет прочитан диапазон.
const FORMAT: u32 = 34;
const FORMAT_MIN: u32 = 15;
const FORMAT_MAX: u32 = 99;

#[derive(Serialize)]
struct Meta {
    pack: MetaBody,
}

#[derive(Serialize)]
struct MetaBody {
    pack_format: u32,
    supported_formats: Range,
    description: String,
}

/// Диапазон записью «от и до»: массив из двух чисел понимают не все версии.
#[derive(Serialize)]
struct Range {
    min_inclusive: u32,
    max_inclusive: u32,
}

#[derive(Serialize)]
struct Font {
    providers: Vec<Provider>,
}

#[derive(Serialize)]
struct Provider {
    #[serde(rename = "type")]
    kind: &'static str,
    file: String,
    /// Насколько глиф поднят над базовой линией. Равен высоте: плашка стоит на
    /// строке ровно так же, как буквы рядом с ней.
    ascent: u32,
    height: u32,
    chars: Vec<String>,
}

/// Собрать пак. Роли без текста пропускаются — рисовать пустую плашку незачем.
pub fn build(roles: &[Role]) -> anyhow::Result<Built> {
    // Время записей постоянное, а не текущее. Иначе одни и те же роли давали бы
    // каждый раз новый zip и новую контрольную сумму: игрок перекачивал бы пак
    // после каждой пересборки, а строка в `server.properties` расходилась бы с
    // файлом в сборке.
    let stamp = zip::DateTime::from_date_and_time(2020, 1, 1, 0, 0, 0)
        .unwrap_or_else(|_| zip::DateTime::default());
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(stamp);
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let mut providers = Vec::new();
    let mut glyphs = Vec::new();

    for role in roles
        .iter()
        .filter(|r| r.image.is_some() || !r.text.trim().is_empty())
    {
        let index = glyphs.len();
        let symbol = codepoint(index);
        let file = format!("assets/noro/textures/font/{}.png", role.name);

        let png = match &role.image {
            Some(bytes) => bytes.clone(),
            None => badge::png(&role.text, &role.color)?,
        };
        // Высота своя у каждой картинки: загруженную вручную разрешено делать
        // кратно выше, чтобы её было чем рисовать, — в игре она ужмётся обратно.
        let height = image::load_from_memory(&png)
            .map(|i| i.height())
            .unwrap_or(badge::height() as u32);

        zip.start_file(&file, options)?;
        zip.write_all(&png)
            .with_context(|| format!("плашка роли {}", role.name))?;

        providers.push(Provider {
            kind: "bitmap",
            file: format!("noro:font/{}.png", role.name),
            ascent: height - 1,
            height,
            chars: vec![symbol.to_string()],
        });
        glyphs.push((role.name.clone(), symbol));
    }

    zip.start_file("assets/noro/font/prefix.json", options)?;
    zip.write_all(&serde_json::to_vec_pretty(&Font { providers })?)?;

    zip.start_file("pack.mcmeta", options)?;
    zip.write_all(&serde_json::to_vec_pretty(&Meta {
        pack: MetaBody {
            pack_format: FORMAT,
            supported_formats: Range {
                min_inclusive: FORMAT_MIN,
                max_inclusive: FORMAT_MAX,
            },
            description: "Noro role prefixes".into(),
        },
    })?)?;

    Ok(Built {
        zip: zip.finish()?.into_inner(),
        glyphs,
    })
}
