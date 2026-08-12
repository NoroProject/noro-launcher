//! 3D изометрический и вращаемый движок рендеринга скинов Minecraft.

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
) -> Vec<u8> {
    let is_legacy = skin.height() == 32;

    let yaw = yaw_deg.to_radians();
    let pitch = pitch_deg.to_radians();
    let cos_y = yaw.cos();
    let sin_y = yaw.sin();
    let cos_p = pitch.cos();
    let sin_p = pitch.sin();

    let is_head_only = mode.contains("head") || mode.contains("cube") || mode.contains("avatar");
    let is_bust = mode.contains("bust") || mode.contains("upper");

    let (cw, ch, center_y) = if is_head_only {
        (22, 22, 0.0)
    } else if is_bust {
        (24, 26, 4.0)
    } else {
        (26, 38, 2.0)
    };

    let cx = cw as f32 / 2.0;
    let cy = ch as f32 / 2.0;

    let mut canvas = ImageBuffer::from_pixel(cw, ch, Rgba([0, 0, 0, 0]));
    let mut z_buf = vec![-1000.0f32; (cw * ch) as usize];

    let project = |x: f32, y: f32, z: f32| -> (i32, i32, f32) {
        let x1 = x * cos_y + z * sin_y;
        let z1 = -x * sin_y + z * cos_y;
        let y2 = y * cos_p - z1 * sin_p;
        let z2 = y * sin_p + z1 * cos_p;

        let sx = (cx + x1).round() as i32;
        let sy = (cy - (y2 - center_y)).round() as i32;
        (sx, sy, z2)
    };

    let mut draw_face = |src_x: u32, src_y: u32, w: u32, h: u32, p0: (f32, f32, f32), du: (f32, f32, f32), dv: (f32, f32, f32), shade: f32| {
        for v in 0..h {
            for u in 0..w {
                let px_x = src_x + u;
                let px_y = src_y + v;
                if px_x >= skin.width() || px_y >= skin.height() { continue; }
                let col = skin.get_pixel(px_x, px_y);
                if col[3] == 0 { continue; }

                let fu = (u as f32 + 0.5) / w as f32;
                let fv = (v as f32 + 0.5) / h as f32;

                let wx = p0.0 + du.0 * fu + dv.0 * fv;
                let wy = p0.1 + du.1 * fu + dv.1 * fv;
                let wz = p0.2 + du.2 * fu + dv.2 * fv;

                let (sx, sy, z_depth) = project(wx, wy, wz);

                if sx >= 0 && sx < cw as i32 && sy >= 0 && sy < ch as i32 {
                    let idx = (sy as u32 * cw + sx as u32) as usize;
                    if z_depth > z_buf[idx] {
                        z_buf[idx] = z_depth;
                        let r = (col[0] as f32 * shade).clamp(0.0, 255.0) as u8;
                        let g = (col[1] as f32 * shade).clamp(0.0, 255.0) as u8;
                        let b = (col[2] as f32 * shade).clamp(0.0, 255.0) as u8;
                        canvas.put_pixel(sx as u32, sy as u32, Rgba([r, g, b, col[3]]));
                    }
                }
            }
        }
    };

    let mut render_cuboid = |pos: (f32, f32, f32), size: (f32, f32, f32), uvs: [(u32, u32, u32, u32); 6], layer_offset: f32| {
        let (x, y, z) = pos;
        let (dx, dy, dz) = size;
        let hx = dx / 2.0 + layer_offset;
        let hy = dy / 2.0 + layer_offset;
        let hz = dz / 2.0 + layer_offset;

        draw_face(uvs[0].0, uvs[0].1, uvs[0].2, uvs[0].3, (x - hx, y + hy, z + hz), (dx + 2.0 * layer_offset, 0.0, 0.0), (0.0, -(dy + 2.0 * layer_offset), 0.0), 1.0);
        draw_face(uvs[1].0, uvs[1].1, uvs[1].2, uvs[1].3, (x + hx, y + hy, z - hz), (-(dx + 2.0 * layer_offset), 0.0, 0.0), (0.0, -(dy + 2.0 * layer_offset), 0.0), 0.75);
        draw_face(uvs[2].0, uvs[2].1, uvs[2].2, uvs[2].3, (x + hx, y + hy, z + hz), (0.0, 0.0, -(dz + 2.0 * layer_offset)), (0.0, -(dy + 2.0 * layer_offset), 0.0), 0.85);
        draw_face(uvs[3].0, uvs[3].1, uvs[3].2, uvs[3].3, (x - hx, y + hy, z - hz), (0.0, 0.0, dz + 2.0 * layer_offset), (0.0, -(dy + 2.0 * layer_offset), 0.0), 0.85);
        draw_face(uvs[4].0, uvs[4].1, uvs[4].2, uvs[4].3, (x - hx, y + hy, z - hz), (dx + 2.0 * layer_offset, 0.0, 0.0), (0.0, 0.0, dz + 2.0 * layer_offset), 1.05);
        draw_face(uvs[5].0, uvs[5].1, uvs[5].2, uvs[5].3, (x - hx, y - hy, z + hz), (dx + 2.0 * layer_offset, 0.0, 0.0), (0.0, 0.0, -(dz + 2.0 * layer_offset)), 0.6);
    };

    let head_pos = if is_head_only { (0.0, 0.0, 0.0) } else { (0.0, 10.0, 0.0) };
    let head_uvs = [(8, 8, 8, 8), (24, 8, 8, 8), (0, 8, 8, 8), (16, 8, 8, 8), (8, 0, 8, 8), (16, 0, 8, 8)];
    render_cuboid(head_pos, (8.0, 8.0, 8.0), head_uvs, 0.0);
    if overlay {
        let hat_uvs = [(40, 8, 8, 8), (56, 8, 8, 8), (32, 8, 8, 8), (48, 8, 8, 8), (40, 0, 8, 8), (48, 0, 8, 8)];
        render_cuboid(head_pos, (8.0, 8.0, 8.0), hat_uvs, 0.35);
    }

    if !is_head_only {
        let torso_uvs = [(20, 20, 8, 12), (32, 20, 8, 12), (16, 20, 4, 12), (28, 20, 4, 12), (20, 16, 8, 4), (28, 16, 8, 4)];
        render_cuboid((0.0, 0.0, 0.0), (8.0, 12.0, 4.0), torso_uvs, 0.0);
        if overlay {
            let jacket_uvs = [(20, 36, 8, 12), (32, 36, 8, 12), (16, 36, 4, 12), (28, 36, 4, 12), (20, 32, 8, 4), (28, 32, 8, 4)];
            render_cuboid((0.0, 0.0, 0.0), (8.0, 12.0, 4.0), jacket_uvs, 0.35);
        }

        let r_arm_uvs = [(44, 20, 4, 12), (52, 20, 4, 12), (40, 20, 4, 12), (48, 20, 4, 12), (44, 16, 4, 4), (48, 16, 4, 4)];
        render_cuboid((-6.0, 0.0, 0.0), (4.0, 12.0, 4.0), r_arm_uvs, 0.0);
        if overlay {
            let r_sleeve_uvs = [(44, 36, 4, 12), (52, 36, 4, 12), (40, 36, 4, 12), (48, 36, 4, 12), (44, 32, 4, 4), (48, 32, 4, 4)];
            render_cuboid((-6.0, 0.0, 0.0), (4.0, 12.0, 4.0), r_sleeve_uvs, 0.35);
        }

        let l_arm_uvs = if !is_legacy {
            [(36, 52, 4, 12), (44, 52, 4, 12), (32, 52, 4, 12), (40, 52, 4, 12), (36, 48, 4, 4), (40, 48, 4, 4)]
        } else {
            [(44, 20, 4, 12), (52, 20, 4, 12), (48, 20, 4, 12), (40, 20, 4, 12), (44, 16, 4, 4), (48, 16, 4, 4)]
        };
        render_cuboid((6.0, 0.0, 0.0), (4.0, 12.0, 4.0), l_arm_uvs, 0.0);
        if overlay && !is_legacy {
            let l_sleeve_uvs = [(52, 52, 4, 12), (60, 52, 4, 12), (48, 52, 4, 12), (56, 52, 4, 12), (52, 48, 4, 4), (56, 48, 4, 4)];
            render_cuboid((6.0, 0.0, 0.0), (4.0, 12.0, 4.0), l_sleeve_uvs, 0.35);
        }

        if !is_bust {
            let r_leg_uvs = [(4, 20, 4, 12), (12, 20, 4, 12), (0, 20, 4, 12), (8, 20, 4, 12), (4, 16, 4, 4), (8, 16, 4, 4)];
            render_cuboid((-2.0, -12.0, 0.0), (4.0, 12.0, 4.0), r_leg_uvs, 0.0);
            if overlay {
                let r_pant_uvs = [(4, 36, 4, 12), (12, 36, 4, 12), (0, 36, 4, 12), (8, 36, 4, 12), (4, 32, 4, 4), (8, 32, 4, 4)];
                render_cuboid((-2.0, -12.0, 0.0), (4.0, 12.0, 4.0), r_pant_uvs, 0.35);
            }

            let l_leg_uvs = if !is_legacy {
                [(20, 52, 4, 12), (28, 52, 4, 12), (16, 52, 4, 12), (24, 52, 4, 12), (20, 48, 4, 4), (24, 48, 4, 4)]
            } else {
                [(4, 20, 4, 12), (12, 20, 4, 12), (8, 20, 4, 12), (0, 20, 4, 12), (4, 16, 4, 4), (8, 16, 4, 4)]
            };
            render_cuboid((2.0, -12.0, 0.0), (4.0, 12.0, 4.0), l_leg_uvs, 0.0);
            if overlay && !is_legacy {
                let l_pant_uvs = [(4, 52, 4, 12), (12, 52, 4, 12), (0, 52, 4, 12), (8, 52, 4, 12), (4, 48, 4, 4), (8, 48, 4, 4)];
                render_cuboid((2.0, -12.0, 0.0), (4.0, 12.0, 4.0), l_pant_uvs, 0.35);
            }
        }
    }

    let final_w = cw * scale;
    let final_h = ch * scale;
    let scaled = DynamicImage::ImageRgba8(canvas).resize(final_w, final_h, FilterType::Nearest);
    let mut bytes = Vec::new();
    let _ = scaled.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png);
    bytes
}
