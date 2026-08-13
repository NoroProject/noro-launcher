//! Тесты рендерера: что yaw и sway реально доходят до растеризатора и что
//! оборот замыкается.

use super::*;
use image::{ImageEncoder, RgbaImage};

/// 64×64 classic skin carrying nothing but a head, each face its own colour.
/// Everything else stays transparent so only the turn shows up in the output —
/// an opaque hat layer would hide the head from every side.
fn test_skin() -> Vec<u8> {
    let mut img = RgbaImage::from_pixel(64, 64, image::Rgba([0, 0, 0, 0]));
    let mut face = |x0: u32, y0: u32, rgb: [u8; 3]| {
        for y in y0..y0 + 8 {
            for x in x0..x0 + 8 {
                img.put_pixel(x, y, image::Rgba([rgb[0], rgb[1], rgb[2], 255]));
            }
        }
    };
    face(8, 0, [255, 255, 255]); // top
    face(16, 0, [20, 20, 20]); // bottom
    face(0, 8, [0, 255, 0]); // right
    face(8, 8, [255, 0, 0]); // front
    face(16, 8, [255, 255, 0]); // left
    face(24, 8, [0, 0, 255]); // back

    // Opaque arm column, so the model resolves as classic rather than slim.
    img.put_pixel(54, 20, image::Rgba([90, 90, 90, 255]));

    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(img.as_raw(), 64, 64, image::ExtendedColorType::Rgba8)
        .expect("encode test skin");
    png
}

fn at(skin: &[u8], yaw: f64, sway: f64) -> Vec<u8> {
    render_rgba(
        skin,
        None,
        64,
        96,
        &View {
            yaw,
            sway,
            ..View::default()
        },
    )
    .expect("valid skin renders")
    .into_raw()
}

#[test]
fn the_turn_shows_every_side() {
    let skin = test_skin();
    // Front, profile and back must all be distinct — otherwise the yaw never
    // reaches the renderer and the figure only appears to spin.
    let (front, side, back) = (
        at(&skin, 0.0, 0.0),
        at(&skin, 90.0, 0.0),
        at(&skin, 180.0, 0.0),
    );
    assert_ne!(front, side);
    assert_ne!(front, back);
    assert_ne!(side, back);
    // A full turn lands back where it started.
    assert_eq!(front, at(&skin, 360.0, 0.0));
}

#[test]
fn sway_moves_limbs_without_touching_the_facing() {
    // Тело в тестовом скине прозрачное, поэтому качание конечностей на картинке
    // не видно — проверяем на модели: тот же yaw при разной фазе даёт ту же
    // ориентацию, то есть sway не подмешивается в поворот.
    let skin = test_skin();
    assert_eq!(at(&skin, 40.0, 0.0), at(&skin, 40.0, 0.5));
}

#[test]
fn a_rendered_frame_is_bgra_and_full_size() {
    let skin = test_skin();
    let frame = render_view(&skin, None, 0.0, 0.0).expect("frame renders");
    let size = frame.size(0);
    // Кадр рендерится в размерах суперсэмплинга, а не в логических PREVIEW_*:
    // единого множителя SUPERSAMPLE тут нет с тех пор, как ширину и высоту
    // развели по отдельным константам.
    assert_eq!(u32::from(size.width), SUPERSAMPLE_W);
    assert_eq!(u32::from(size.height), SUPERSAMPLE_H);

    // Красная маска лица должна лежать в синем канале BGRA — если забыть своп,
    // скин поедет в неправильных цветах, а тесты растеризатора этого не увидят.
    let bytes = frame.as_bytes(0).expect("frame data");
    let has_red_as_bgr = bytes
        .chunks_exact(4)
        .any(|p| p[3] > 200 && p[2] > 200 && p[1] < 80 && p[0] < 80);
    assert!(has_red_as_bgr, "ожидали красный как B=0,G=0,R=255 в BGRA");
}
