//! 3D изометрический движок рендеринга скинов (по мотивам PandoraLauncher).

use super::skin_render_math::{rasterize_quad, Mat3, V3};
use super::skin_render_parts::{build_quads, BodyPart};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use std::io::Cursor;

pub fn render_3d(
    skin: &DynamicImage,
    scale: u32,
    overlay: bool,
    mode: &str,
    yaw_deg: f32,
    pitch_deg: f32,
    sway: f32,
) -> Vec<u8> {
    let is_legacy = skin.height() == 32;
    let is_slim = skin.width() == 64 && skin.get_pixel(54, 20)[3] == 0;

    let ry = (yaw_deg as f64).to_radians();
    let rx = (pitch_deg as f64).to_radians();
    let mat = Mat3::rotation_yx(ry, rx);

    let is_head_only = mode.contains("head") || mode.contains("cube") || mode.contains("avatar");
    let is_bust = mode.contains("bust") || mode.contains("upper");

    let (cw, ch, center_y) = if is_head_only {
        (22, 22, 0.0)
    } else if is_bust {
        (24, 26, 4.0)
    } else {
        (26, 38, 2.0)
    };

    let arm_w = if is_slim { 3.0 } else { 4.0 };
    let arm_r_x = -4.0 - arm_w;

    let sway_rad = (sway as f64 * 360.0).to_radians();
    let r_arm_pitch = (sway_rad).sin() * 0.45;
    let l_arm_pitch = -(sway_rad).sin() * 0.45;
    let r_leg_pitch = -(sway_rad).sin() * 0.45;
    let l_leg_pitch = (sway_rad).sin() * 0.45;

    let mut parts = Vec::new();

    let head_y = if is_head_only { 0.0 } else { 10.0 };
    parts.push(BodyPart { min: V3::new(-4.0, head_y - 4.0, -4.0), max: V3::new(4.0, head_y + 4.0, 4.0), pivot: V3::new(0.0, head_y, 0.0), tx: 0.0, ty: 0.0, is_slim: false, is_overlay: false, rot: (0.0, 0.0) });
    if overlay {
        parts.push(BodyPart { min: V3::new(-4.0, head_y - 4.0, -4.0), max: V3::new(4.0, head_y + 4.0, 4.0), pivot: V3::new(0.0, head_y, 0.0), tx: 32.0, ty: 0.0, is_slim: false, is_overlay: true, rot: (0.0, 0.0) });
    }

    if !is_head_only {
        parts.push(BodyPart { min: V3::new(-4.0, -6.0, -2.0), max: V3::new(4.0, 6.0, 2.0), pivot: V3::new(0.0, 0.0, 0.0), tx: 16.0, ty: 16.0, is_slim: false, is_overlay: false, rot: (0.0, 0.0) });
        if overlay {
            parts.push(BodyPart { min: V3::new(-4.0, -6.0, -2.0), max: V3::new(4.0, 6.0, 2.0), pivot: V3::new(0.0, 0.0, 0.0), tx: 16.0, ty: 32.0, is_slim: false, is_overlay: true, rot: (0.0, 0.0) });
        }

        parts.push(BodyPart { min: V3::new(arm_r_x, -6.0, -2.0), max: V3::new(-4.0, 6.0, 2.0), pivot: V3::new(-4.0, 4.0, 0.0), tx: 40.0, ty: 16.0, is_slim, is_overlay: false, rot: (r_arm_pitch, 0.0) });
        if overlay {
            parts.push(BodyPart { min: V3::new(arm_r_x, -6.0, -2.0), max: V3::new(-4.0, 6.0, 2.0), pivot: V3::new(-4.0, 4.0, 0.0), tx: 40.0, ty: 32.0, is_slim, is_overlay: true, rot: (r_arm_pitch, 0.0) });
        }

        let (l_tx, l_ty_ov) = if !is_legacy { (32.0, 48.0) } else { (40.0, 32.0) };
        parts.push(BodyPart { min: V3::new(4.0, -6.0, -2.0), max: V3::new(4.0 + arm_w, 6.0, 2.0), pivot: V3::new(4.0, 4.0, 0.0), tx: l_tx, ty: 48.0, is_slim, is_overlay: false, rot: (l_arm_pitch, 0.0) });
        if overlay && !is_legacy {
            parts.push(BodyPart { min: V3::new(4.0, -6.0, -2.0), max: V3::new(4.0 + arm_w, 6.0, 2.0), pivot: V3::new(4.0, 4.0, 0.0), tx: 48.0, ty: l_ty_ov, is_slim, is_overlay: true, rot: (l_arm_pitch, 0.0) });
        }

        if !is_bust {
            parts.push(BodyPart { min: V3::new(-4.0, -18.0, -2.0), max: V3::new(0.0, -6.0, 2.0), pivot: V3::new(-2.0, -6.0, 0.0), tx: 0.0, ty: 16.0, is_slim: false, is_overlay: false, rot: (r_leg_pitch, 0.0) });
            if overlay {
                parts.push(BodyPart { min: V3::new(-4.0, -18.0, -2.0), max: V3::new(0.0, -6.0, 2.0), pivot: V3::new(-2.0, -6.0, 0.0), tx: 0.0, ty: 32.0, is_slim: false, is_overlay: true, rot: (r_leg_pitch, 0.0) });
            }

            let l_leg_tx = if !is_legacy { 16.0 } else { 0.0 };
            parts.push(BodyPart { min: V3::new(0.0, -18.0, -2.0), max: V3::new(4.0, -6.0, 2.0), pivot: V3::new(2.0, -6.0, 0.0), tx: l_leg_tx, ty: 48.0, is_slim: false, is_overlay: false, rot: (l_leg_pitch, 0.0) });
            if overlay && !is_legacy {
                parts.push(BodyPart { min: V3::new(0.0, -18.0, -2.0), max: V3::new(4.0, -6.0, 2.0), pivot: V3::new(2.0, -6.0, 0.0), tx: 0.0, ty: 48.0, is_slim: false, is_overlay: true, rot: (l_leg_pitch, 0.0) });
            }
        }
    }

    let quads = build_quads(&parts);

    let cx = cw as f64 / 2.0;
    let cy = ch as f64 / 2.0;

    let mut canvas = ImageBuffer::from_pixel(cw, ch, Rgba([0, 0, 0, 0]));
    let mut z_buf = vec![-1000.0f64; (cw * ch) as usize];

    for quad in &quads {
        rasterize_quad(quad, &mat, skin, &mut canvas, &mut z_buf, cw, ch, cx, cy, center_y);
    }

    let final_w = cw * scale;
    let final_h = ch * scale;
    let scaled = DynamicImage::ImageRgba8(canvas).resize(final_w, final_h, FilterType::Nearest);
    let mut bytes = Vec::new();
    let _ = scaled.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png);
    bytes
}
