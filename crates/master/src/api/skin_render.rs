//! Движок генерации 2D и 3D рендеров скинов и плащей Minecraft.

use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use std::io::Cursor;

pub fn render_head(skin: &DynamicImage, scale: u32, overlay: bool) -> Vec<u8> {
    let mut head = ImageBuffer::new(8, 8);
    copy_region(skin, &mut head, 8, 8, 8, 8, 0, 0);
    if overlay {
        copy_region_alpha(skin, &mut head, 40, 8, 8, 8, 0, 0);
    }
    scale_and_encode(&DynamicImage::ImageRgba8(head), 8 * scale, 8 * scale)
}

pub fn render_bust(skin: &DynamicImage, scale: u32, overlay: bool) -> Vec<u8> {
    let is_legacy = skin.height() == 32;
    let mut canvas = ImageBuffer::from_pixel(16, 20, Rgba([0, 0, 0, 0]));

    copy_region(skin, &mut canvas, 8, 8, 8, 8, 4, 0);
    if overlay { copy_region_alpha(skin, &mut canvas, 40, 8, 8, 8, 4, 0); }

    copy_region(skin, &mut canvas, 20, 20, 8, 12, 4, 8);
    if overlay { copy_region_alpha(skin, &mut canvas, 20, 36, 8, 12, 4, 8); }

    copy_region(skin, &mut canvas, 44, 20, 4, 12, 0, 8);
    if overlay { copy_region_alpha(skin, &mut canvas, 44, 36, 4, 12, 0, 8); }

    if !is_legacy {
        copy_region(skin, &mut canvas, 36, 52, 4, 12, 12, 8);
        if overlay { copy_region_alpha(skin, &mut canvas, 52, 52, 4, 12, 12, 8); }
    } else {
        copy_region(skin, &mut canvas, 44, 20, 4, 12, 12, 8);
    }

    scale_and_encode(&DynamicImage::ImageRgba8(canvas), 16 * scale, 20 * scale)
}

pub fn render_body(skin: &DynamicImage, scale: u32, overlay: bool) -> Vec<u8> {
    let is_legacy = skin.height() == 32;
    let mut canvas = ImageBuffer::from_pixel(16, 32, Rgba([0, 0, 0, 0]));

    copy_region(skin, &mut canvas, 8, 8, 8, 8, 4, 0);
    if overlay { copy_region_alpha(skin, &mut canvas, 40, 8, 8, 8, 4, 0); }

    copy_region(skin, &mut canvas, 20, 20, 8, 12, 4, 8);
    if overlay { copy_region_alpha(skin, &mut canvas, 20, 36, 8, 12, 4, 8); }

    copy_region(skin, &mut canvas, 44, 20, 4, 12, 0, 8);
    if overlay { copy_region_alpha(skin, &mut canvas, 44, 36, 4, 12, 0, 8); }

    if !is_legacy {
        copy_region(skin, &mut canvas, 36, 52, 4, 12, 12, 8);
        if overlay { copy_region_alpha(skin, &mut canvas, 52, 52, 4, 12, 12, 8); }
    } else {
        copy_region(skin, &mut canvas, 44, 20, 4, 12, 12, 8);
    }

    copy_region(skin, &mut canvas, 4, 20, 4, 12, 4, 20);
    if overlay { copy_region_alpha(skin, &mut canvas, 4, 36, 4, 12, 4, 20); }

    if !is_legacy {
        copy_region(skin, &mut canvas, 20, 52, 4, 12, 8, 20);
        if overlay { copy_region_alpha(skin, &mut canvas, 4, 52, 4, 12, 8, 20); }
    } else {
        copy_region(skin, &mut canvas, 4, 20, 4, 12, 8, 20);
    }

    scale_and_encode(&DynamicImage::ImageRgba8(canvas), 16 * scale, 32 * scale)
}

pub fn render_cube_head(skin: &DynamicImage, scale: u32, overlay: bool) -> Vec<u8> {
    let mut canvas = ImageBuffer::from_pixel(18, 20, Rgba([0, 0, 0, 0]));

    for y in 0..8 {
        for x in 0..8 {
            let px = (9 + x - y).clamp(0, 17);
            let py = (6 + (x + y) / 2).clamp(0, 19);
            let p = skin.get_pixel(8 + (7 - x), 8 + y);
            if p[3] > 0 { canvas.put_pixel(px, py, p); }
            if overlay {
                let o = skin.get_pixel(40 + (7 - x), 8 + y);
                if o[3] > 0 { canvas.put_pixel(px, py, o); }
            }
        }
    }

    for y in 0..8 {
        for x in 0..8 {
            let px = (1 + x + y).clamp(0, 17);
            let py = (10 + y - x / 2).clamp(0, 19);
            let p = skin.get_pixel(x, 8 + y);
            if p[3] > 0 { canvas.put_pixel(px, py, p); }
            if overlay {
                let o = skin.get_pixel(32 + x, 8 + y);
                if o[3] > 0 { canvas.put_pixel(px, py, o); }
            }
        }
    }

    scale_and_encode(&DynamicImage::ImageRgba8(canvas), 18 * scale, 20 * scale)
}

pub fn render_cape(cape_img: &DynamicImage, scale: u32) -> Vec<u8> {
    let mut canvas = ImageBuffer::from_pixel(10, 16, Rgba([0, 0, 0, 0]));
    copy_region(cape_img, &mut canvas, 1, 1, 10, 16, 0, 0);
    scale_and_encode(&DynamicImage::ImageRgba8(canvas), 10 * scale, 16 * scale)
}

fn copy_region(src: &DynamicImage, dst: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, sx: u32, sy: u32, w: u32, h: u32, dx: u32, dy: u32) {
    for y in 0..h {
        for x in 0..w {
            if sx + x < src.width() && sy + y < src.height() {
                dst.put_pixel(dx + x, dy + y, src.get_pixel(sx + x, sy + y));
            }
        }
    }
}

fn copy_region_alpha(src: &DynamicImage, dst: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, sx: u32, sy: u32, w: u32, h: u32, dx: u32, dy: u32) {
    for y in 0..h {
        for x in 0..w {
            if sx + x < src.width() && sy + y < src.height() {
                let p = src.get_pixel(sx + x, sy + y);
                if p[3] > 0 {
                    dst.put_pixel(dx + x, dy + y, p);
                }
            }
        }
    }
}

fn scale_and_encode(img: &DynamicImage, w: u32, h: u32) -> Vec<u8> {
    let scaled = img.resize(w, h, FilterType::Nearest);
    let mut bytes = Vec::new();
    let _ = scaled.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png);
    bytes
}
