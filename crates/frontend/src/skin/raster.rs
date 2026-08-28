//! Textured triangle rasterizer with a z-buffer.

use super::math::V3;
use super::project::ProjectedQuad;
use image::{DynamicImage, GenericImageView, Pixel, RgbaImage};

pub struct Texture<'a> {
    pub image: &'a DynamicImage,
    /// Texture pixels per one 64×64 layout pixel (2.0 for a 128px HD skin).
    pub uv_scale: f64,
}

/// 2D edge function: positive when `p` lies left of the edge `a→b`.
#[inline]
fn edge(a: V3, b: V3, px: f64, py: f64) -> f64 {
    (b.x - a.x) * (py - a.y) - (b.y - a.y) * (px - a.x)
}

pub fn rasterize_quad(quad: &ProjectedQuad, tex: &Texture, out: &mut RgbaImage, zbuf: &mut [f64]) {
    triangle(quad, tex, out, zbuf, 0, 1, 2);
    triangle(quad, tex, out, zbuf, 0, 2, 3);
}

fn triangle(
    quad: &ProjectedQuad,
    tex: &Texture,
    out: &mut RgbaImage,
    zbuf: &mut [f64],
    i0: usize,
    i1: usize,
    i2: usize,
) {
    let (v0, v1, v2) = (quad.verts[i0], quad.verts[i1], quad.verts[i2]);
    let (uv0, uv1, uv2) = (quad.uvs[i0], quad.uvs[i1], quad.uvs[i2]);

    let area = edge(v0, v1, v2.x, v2.y);
    if area.abs() < 0.001 {
        return; // degenerate
    }
    let inv_area = 1.0 / area;

    let (out_w, out_h) = (out.width(), out.height());
    let min_x = v0.x.min(v1.x).min(v2.x).floor().max(0.0) as i32;
    let max_x = v0.x.max(v1.x).max(v2.x).ceil().min(out_w as f64 - 1.0) as i32;
    let min_y = v0.y.min(v1.y).min(v2.y).floor().max(0.0) as i32;
    let max_y = v0.y.max(v1.y).max(v2.y).ceil().min(out_h as f64 - 1.0) as i32;

    let (tex_w, tex_h) = (tex.image.width(), tex.image.height());

    for py in min_y..=max_y {
        for px in min_x..=max_x {
            let (cx, cy) = (px as f64 + 0.5, py as f64 + 0.5);
            let w0 = edge(v1, v2, cx, cy) * inv_area;
            let w1 = edge(v2, v0, cx, cy) * inv_area;
            let w2 = 1.0 - w0 - w1;
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }

            let z = w0 * v0.z + w1 * v1.z + w2 * v2.z;
            let idx = (py as u32 * out_w + px as u32) as usize;
            if z <= zbuf[idx] {
                continue;
            }

            let u = (w0 * uv0.0 + w1 * uv1.0 + w2 * uv2.0) * tex.uv_scale;
            let v = (w0 * uv0.1 + w1 * uv1.1 + w2 * uv2.1) * tex.uv_scale;
            let tx = (u.floor().max(0.0) as u32).min(tex_w - 1);
            let ty = (v.floor().max(0.0) as u32).min(tex_h - 1);

            let mut pixel = tex.image.get_pixel(tx, ty);
            for c in 0..3 {
                pixel[c] = ((pixel[c] as u16 * quad.shade as u16) / 255) as u8;
            }

            if !quad.allow_transparency {
                pixel[3] = 0xFF;
                out.put_pixel(px as u32, py as u32, pixel);
                zbuf[idx] = z;
            } else if pixel[3] > 0 {
                out.get_pixel_mut(px as u32, py as u32).blend(&pixel);
                zbuf[idx] = z;
            }
        }
    }
}
