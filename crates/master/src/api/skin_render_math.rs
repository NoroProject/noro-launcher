//! 3D математика, геометрия и высокоточная растеризация скинов (по мотивам PandoraLauncher).

use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

#[derive(Clone, Copy, Debug)]
pub struct V3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl V3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    pub fn dot(&self, other: V3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

pub struct Mat3(pub [[f64; 3]; 3]);

impl Mat3 {
    pub fn rotation_yx(ry: f64, rx: f64) -> Self {
        let (sy, cy) = ry.sin_cos();
        let (sx, cx) = rx.sin_cos();
        Self([
            [cy, 0.0, sy],
            [sx * sy, cx, -sx * cy],
            [-cx * sy, sx, cx * cy],
        ])
    }

    pub fn transform(&self, v: V3) -> V3 {
        let r = &self.0;
        V3 {
            x: r[0][0] * v.x + r[0][1] * v.y + r[0][2] * v.z,
            y: r[1][0] * v.x + r[1][1] * v.y + r[1][2] * v.z,
            z: r[2][0] * v.x + r[2][1] * v.y + r[2][2] * v.z,
        }
    }
}

pub struct Quad {
    pub verts: [V3; 4],
    pub uvs: [(f64, f64); 4],
    pub normal: V3,
    pub is_overlay: bool,
}

/// Куда рисуем: размеры холста, центр проекции и масштаб.
///
/// Шесть чисел, которые всегда едут вместе и по отдельности ничего не значат.
/// Пока они были отдельными аргументами, вызов растеризатора состоял из
/// одиннадцати позиций, и перепутать `cx` с `cy` можно было не заметив.
#[derive(Clone, Copy)]
pub struct Canvas {
    pub w: u32,
    pub h: u32,
    /// Центр проекции в пикселях холста.
    pub cx: f64,
    pub cy: f64,
    /// Высота, относительно которой считается вертикаль модели.
    pub center_y: f64,
    /// Во сколько раз холст крупнее итоговой картинки.
    pub res_mult: f64,
}

pub fn rasterize_quad_highres(
    quad: &Quad,
    mat: &Mat3,
    skin: &DynamicImage,
    canvas: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    z_buf: &mut [f64],
    view: Canvas,
) {
    let Canvas {
        w: cw,
        h: ch,
        cx,
        cy,
        center_y,
        res_mult,
    } = view;
    let light = V3::new(0.3, 0.8, 0.5);
    let trans_norm = mat.transform(quad.normal);
    let l_dot = trans_norm.dot(light).max(0.0);
    let shade = 0.65 + 0.35 * l_dot;

    let tv: [V3; 4] = [
        mat.transform(quad.verts[0]),
        mat.transform(quad.verts[1]),
        mat.transform(quad.verts[2]),
        mat.transform(quad.verts[3]),
    ];

    let min_x = tv.iter().map(|v| v.x).fold(f64::INFINITY, f64::min);
    let max_x = tv.iter().map(|v| v.x).fold(f64::NEG_INFINITY, f64::max);
    let min_y = tv.iter().map(|v| v.y).fold(f64::INFINITY, f64::min);
    let max_y = tv.iter().map(|v| v.y).fold(f64::NEG_INFINITY, f64::max);

    let sx_min = ((cx + min_x * res_mult) - 2.0).floor().max(0.0) as u32;
    let sx_max = ((cx + max_x * res_mult) + 2.0).ceil().min(cw as f64 - 1.0) as u32;
    let sy_min = ((cy - (max_y - center_y) * res_mult) - 2.0)
        .floor()
        .max(0.0) as u32;
    let sy_max = ((cy - (min_y - center_y) * res_mult) + 2.0)
        .ceil()
        .min(ch as f64 - 1.0) as u32;

    let z_bias = if quad.is_overlay { 0.08 } else { 0.0 };

    for py in sy_min..=sy_max {
        for px in sx_min..=sx_max {
            let wx = (px as f64 - cx) / res_mult;
            let wy = (cy - py as f64) / res_mult + center_y;

            if let Some((u, v, z_depth)) = sample_barycentric(wx, wy, &tv, &quad.uvs) {
                let skin_x = u.floor().max(0.0) as u32;
                let skin_y = v.floor().max(0.0) as u32;
                if skin_x >= skin.width() || skin_y >= skin.height() {
                    continue;
                }

                let col = skin.get_pixel(skin_x, skin_y);
                if col[3] == 0 {
                    continue;
                }

                let idx = (py * cw + px) as usize;
                let effective_z = z_depth + z_bias;
                if effective_z > z_buf[idx] {
                    z_buf[idx] = effective_z;
                    let r = (col[0] as f64 * shade).clamp(0.0, 255.0) as u8;
                    let g = (col[1] as f64 * shade).clamp(0.0, 255.0) as u8;
                    let b = (col[2] as f64 * shade).clamp(0.0, 255.0) as u8;
                    canvas.put_pixel(px, py, Rgba([r, g, b, col[3]]));
                }
            }
        }
    }
}

fn sample_barycentric(
    x: f64,
    y: f64,
    verts: &[V3; 4],
    uvs: &[(f64, f64); 4],
) -> Option<(f64, f64, f64)> {
    // Квад — это два треугольника: 0-1-2 и 0-2-3. Точка попадает ровно в один.
    if let Some(hit) = tri_bary(
        x,
        y,
        [verts[0], verts[1], verts[2]],
        [uvs[0], uvs[1], uvs[2]],
    ) {
        return Some(hit);
    }
    tri_bary(
        x,
        y,
        [verts[0], verts[2], verts[3]],
        [uvs[0], uvs[2], uvs[3]],
    )
}

/// Барицентрические координаты точки в треугольнике.
///
/// Вершины и их UV приходят массивами, а не шестью аргументами подряд: три пары
/// «точка и её текстурная координата» обязаны совпадать по порядку, и разложенные
/// в плоский список они разъезжались при любой правке.
fn tri_bary(x: f64, y: f64, p: [V3; 3], uv: [(f64, f64); 3]) -> Option<(f64, f64, f64)> {
    let ([p0, p1, p2], [u0, u1, u2]) = (p, uv);
    let den = (p1.y - p2.y) * (p0.x - p2.x) + (p2.x - p1.x) * (p0.y - p2.y);
    if den.abs() < 1e-6 {
        return None;
    }
    let w0 = ((p1.y - p2.y) * (x - p2.x) + (p2.x - p1.x) * (y - p2.y)) / den;
    let w1 = ((p2.y - p0.y) * (x - p2.x) + (p0.x - p2.x) * (y - p2.y)) / den;
    let w2 = 1.0 - w0 - w1;
    if w0 >= -0.001 && w1 >= -0.001 && w2 >= -0.001 {
        let u = w0 * u0.0 + w1 * u1.0 + w2 * u2.0;
        let v = w0 * u0.1 + w1 * u1.1 + w2 * u2.1;
        let z = w0 * p0.z + w1 * p1.z + w2 * p2.z;
        Some((u, v, z))
    } else {
        None
    }
}
