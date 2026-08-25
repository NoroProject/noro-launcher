use super::*;
use std::io::Read;

fn role(name: &str, text: &str, color: &str) -> Role {
    Role {
        name: name.into(),
        text: text.into(),
        color: color.into(),
        image: None,
    }
}

fn entries(zip: &[u8]) -> Vec<String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip.to_vec())).unwrap();
    (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect()
}

fn read(zip: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip.to_vec())).unwrap();
    let mut out = String::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    out
}

#[test]
fn packs_one_texture_per_role() {
    let built = build(&[
        role("admin", "АДМИН", "#ff8c82"),
        role("friend", "FRIEND", "#f3c969"),
    ])
    .unwrap();

    let names = entries(&built.zip);
    assert!(
        names.contains(&"assets/noro/textures/font/admin.png".to_string()),
        "{names:?}"
    );
    assert!(
        names.contains(&"assets/noro/textures/font/friend.png".to_string()),
        "{names:?}"
    );
    assert!(
        names.contains(&"assets/noro/font/prefix.json".to_string()),
        "{names:?}"
    );
    assert!(names.contains(&"pack.mcmeta".to_string()), "{names:?}");
}

/// Роли получают разные символы: один на всех превратил бы плашки в одну.
#[test]
fn gives_every_role_its_own_codepoint() {
    let built = build(&[
        role("admin", "АДМИН", "#ff8c82"),
        role("friend", "FRIEND", "#f3c969"),
    ])
    .unwrap();

    assert_eq!(built.glyphs.len(), 2);
    assert_eq!(built.glyphs[0].1, '\u{E000}');
    assert_eq!(built.glyphs[1].1, '\u{E001}');

    let font = read(&built.zip, "assets/noro/font/prefix.json");
    assert!(font.contains("noro:font/admin.png"), "{font}");
    assert!(font.contains("bitmap"), "{font}");
}

/// Роль без текста плашки не получает: пустой прямоугольник — не префикс.
#[test]
fn skips_roles_without_text() {
    let built = build(&[
        role("player", "  ", "#9CA3AF"),
        role("admin", "АДМИН", "#ff8c82"),
    ])
    .unwrap();

    assert_eq!(built.glyphs.len(), 1);
    assert_eq!(built.glyphs[0].0, "admin");
}

/// Плашка — настоящий PNG, и её ширина растёт вместе с текстом.
#[test]
fn draws_a_png_that_grows_with_the_text() {
    let short = badge::png("ОК", "#ff8c82").unwrap();
    let long = badge::png("АДМИНИСТРАТОР", "#ff8c82").unwrap();

    assert_eq!(&short[..8], b"\x89PNG\r\n\x1a\n");
    let size = |png: &[u8]| u32::from_be_bytes(png[16..20].try_into().unwrap());
    assert!(size(&long) > size(&short), "длинный текст — шире плашка");

    // Высота фиксированная: значок должен сидеть на строке чата.
    let height = u32::from_be_bytes(long[20..24].try_into().unwrap());
    assert_eq!(height, badge::height() as u32);
}

/// Мусор в поле цвета не роняет сборку пака: роль просто станет серой.
#[test]
fn survives_a_broken_color() {
    assert_eq!(badge::rgb("#ff8c82"), (0xff, 0x8c, 0x82));
    assert_eq!(badge::rgb("ff8c82"), (0xff, 0x8c, 0x82));
    assert_eq!(badge::rgb("зелёный"), (0x9C, 0xA3, 0xAF));
    assert!(badge::png("АДМИН", "").is_ok());
}

/// Не проверка, а глаза: складывает образцы плашек, чтобы на них посмотреть.
#[test]
#[ignore]
fn dump_samples() {
    for (name, text, color) in [
        ("admin", "АДМИН", "#ff8c82"),
        ("friend", "FRIEND", "#f3c969"),
        ("mod", "МОДЕР 1", "#7FB2FF"),
        ("ref", "ADMIN", "#e52601"),
    ] {
        let png = badge::png(text, color).unwrap();
        std::fs::write(format!("/tmp/badge-{name}.png"), png).unwrap();
    }
}

/// Геометрия снята с эталона: «ADMIN» — ровно 25×7, по пикселю поля с краёв.
#[test]
fn matches_the_reference_geometry() {
    let png = badge::png("ADMIN", "#e52601").unwrap();
    let size = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().unwrap());

    assert_eq!(size(16), 25, "ширина");
    assert_eq!(size(20), 7, "высота");
}

/// Тень сдвинута только вправо.
///
/// Со сдвигом вниз она вылезала в нижнюю строку плашки, которой полагается быть
/// чистым фоном, — у эталона там ни одного тёмного пикселя.
#[test]
fn keeps_the_shadow_on_the_same_row() {
    let png = badge::png("I", "#3355ff").unwrap();
    let img = image::load_from_memory(&png).unwrap().to_rgba8();

    let dark = |y: u32| {
        (0..img.width())
            .filter(|x| {
                let p = img.get_pixel(*x, y);
                let lum = 0.299 * p.0[0] as f32 + 0.587 * p.0[1] as f32 + 0.114 * p.0[2] as f32;
                lum < 40.0
            })
            .count()
    };

    // Буква занимает строки 1..5, значит тень — тоже они и только они.
    assert_eq!(dark(0), 0, "верхняя строка плашки — фон");
    assert_eq!(dark(6), 0, "нижняя строка плашки — фон");
}

/// Коды цвета из префикса на плашку не попадают: там цвет задаёт градиент.
#[test]
fn strips_color_codes_from_the_label() {
    assert_eq!(super::store::strip_codes("§x§f§f§8§c§8§2АДМИН§r"), "АДМИН");
    assert_eq!(super::store::strip_codes("&cГЛАВА"), "ГЛАВА");
    assert_eq!(super::store::strip_codes("#ff8c82VIP"), "VIP");
    assert_eq!(super::store::strip_codes("АДМИН"), "АДМИН");
}

/// Одни и те же роли дают один и тот же файл.
///
/// Пак раздаётся по контрольной сумме: плавай она от сборки к сборке — игрок
/// качал бы его заново после каждого запроса, а `server.properties` расходился
/// бы со сборкой.
#[test]
fn builds_byte_for_byte_the_same_pack() {
    let roles = || {
        vec![
            role("admin", "АДМИН", "#ff8c82"),
            role("friend", "FRIEND", "#f3c969"),
        ]
    };

    assert_eq!(build(&roles()).unwrap().zip, build(&roles()).unwrap().zip);
}

/// Загруженная картинка заменяет нарисованную целиком.
#[test]
fn prefers_an_uploaded_image() {
    let mine = badge::png("СВОЁ", "#3355ff").unwrap();
    let built = build(&[Role {
        name: "admin".into(),
        text: "АДМИН".into(),
        color: "#ff8c82".into(),
        image: Some(mine.clone()),
    }])
    .unwrap();

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(built.zip)).unwrap();
    let mut out = Vec::new();
    std::io::Read::read_to_end(
        &mut archive
            .by_name("assets/noro/textures/font/admin.png")
            .unwrap(),
        &mut out,
    )
    .unwrap();

    assert_eq!(out, mine, "в паке лежит именно загруженное");
}

/// Роль без текста, но с картинкой, плашку получает: картинка и есть плашка.
#[test]
fn keeps_a_role_that_has_only_an_image() {
    let built = build(&[Role {
        name: "vip".into(),
        text: "  ".into(),
        color: "#ffffff".into(),
        image: Some(badge::png("V", "#ffcc00").unwrap()),
    }])
    .unwrap();

    assert_eq!(built.glyphs.len(), 1);
}

/// Критерии размера: высота кратна строке, ширина в разумных пределах.
#[test]
fn checks_the_uploaded_size() {
    use crate::api::admin::role_badge::measure;

    let line = badge::height() as u32;
    let good = badge::png("OK", "#3355ff").unwrap();
    assert_eq!(measure(&good).unwrap().1, line);

    // Не кратная строке — не сядет в чат ровно.
    let mut odd = image::RgbaImage::new(10, line + 1);
    odd.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
    let mut bytes = Vec::new();
    odd.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();
    assert!(measure(&bytes).is_err());

    assert!(measure(b"not a png").is_err());
}
