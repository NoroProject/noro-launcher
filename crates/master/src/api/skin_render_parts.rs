//! Построение полигональной модели персонажа Minecraft с идеальным UV-маппингом (по образцу PandoraLauncher).

use super::skin_render_math::{Mat3, Quad, V3};

pub struct BodyPart {
    pub min: V3,
    pub max: V3,
    pub pivot: V3,
    pub tx: f64,
    pub ty: f64,
    pub is_slim: bool,
    pub is_overlay: bool,
    pub rot: (f64, f64), // (pitch, roll/yaw)
}

pub fn build_quads(parts: &[BodyPart]) -> Vec<Quad> {
    let mut quads = Vec::new();
    for p in parts {
        let tex_w = p.max.x - p.min.x;
        let tex_h = p.max.y - p.min.y;
        let tex_d = p.max.z - p.min.z;

        let inf = if p.is_overlay { 0.25 } else { 0.0 };
        let min = V3::new(p.min.x - inf, p.min.y - inf, p.min.z - inf);
        let max = V3::new(p.max.x + inf, p.max.y + inf, p.max.z + inf);

        let tx = p.tx;
        let ty = p.ty;
        let w = tex_w;
        let h = tex_h;
        let d = tex_d;

        let local_quads = [
            // Front (+Z)
            Quad {
                verts: [
                    V3::new(min.x, max.y, max.z),
                    V3::new(max.x, max.y, max.z),
                    V3::new(max.x, min.y, max.z),
                    V3::new(min.x, min.y, max.z),
                ],
                uvs: [
                    (tx + d, ty + d),
                    (tx + d + w, ty + d),
                    (tx + d + w, ty + d + h),
                    (tx + d, ty + d + h),
                ],
                normal: V3::new(0.0, 0.0, 1.0),
                is_overlay: p.is_overlay,
            },
            // Back (-Z)
            Quad {
                verts: [
                    V3::new(max.x, max.y, min.z),
                    V3::new(min.x, max.y, min.z),
                    V3::new(min.x, min.y, min.z),
                    V3::new(max.x, min.y, min.z),
                ],
                uvs: [
                    (tx + 2.0 * d + w, ty + d),
                    (tx + 2.0 * d + 2.0 * w, ty + d),
                    (tx + 2.0 * d + 2.0 * w, ty + d + h),
                    (tx + 2.0 * d + w, ty + d + h),
                ],
                normal: V3::new(0.0, 0.0, -1.0),
                is_overlay: p.is_overlay,
            },
            // Right (+X)
            Quad {
                verts: [
                    V3::new(max.x, max.y, max.z),
                    V3::new(max.x, max.y, min.z),
                    V3::new(max.x, min.y, min.z),
                    V3::new(max.x, min.y, max.z),
                ],
                uvs: [
                    (tx + d, ty + d),
                    (tx, ty + d),
                    (tx, ty + d + h),
                    (tx + d, ty + d + h),
                ],
                normal: V3::new(1.0, 0.0, 0.0),
                is_overlay: p.is_overlay,
            },
            // Left (-X)
            Quad {
                verts: [
                    V3::new(min.x, max.y, min.z),
                    V3::new(min.x, max.y, max.z),
                    V3::new(min.x, min.y, max.z),
                    V3::new(min.x, min.y, min.z),
                ],
                uvs: [
                    (tx + 2.0 * d + w, ty + d),
                    (tx + d + w, ty + d),
                    (tx + d + w, ty + d + h),
                    (tx + 2.0 * d + w, ty + d + h),
                ],
                normal: V3::new(-1.0, 0.0, 0.0),
                is_overlay: p.is_overlay,
            },
            // Top (+Y)
            Quad {
                verts: [
                    V3::new(min.x, max.y, min.z),
                    V3::new(max.x, max.y, min.z),
                    V3::new(max.x, max.y, max.z),
                    V3::new(min.x, max.y, max.z),
                ],
                uvs: [
                    (tx + d, ty),
                    (tx + d + w, ty),
                    (tx + d + w, ty + d),
                    (tx + d, ty + d),
                ],
                normal: V3::new(0.0, 1.0, 0.0),
                is_overlay: p.is_overlay,
            },
            // Bottom (-Y)
            Quad {
                verts: [
                    V3::new(min.x, min.y, max.z),
                    V3::new(max.x, min.y, max.z),
                    V3::new(max.x, min.y, min.z),
                    V3::new(min.x, min.y, min.z),
                ],
                uvs: [
                    (tx + d + w, ty),
                    (tx + 2.0 * d + w, ty),
                    (tx + 2.0 * d + w, ty + d),
                    (tx + d + w, ty + d),
                ],
                normal: V3::new(0.0, -1.0, 0.0),
                is_overlay: p.is_overlay,
            },
        ];

        let rot_mat = Mat3::rotation_yx(p.rot.1, p.rot.0);

        for mut q in local_quads {
            for v in &mut q.verts {
                let rel = V3::new(v.x - p.pivot.x, v.y - p.pivot.y, v.z - p.pivot.z);
                let r = rot_mat.transform(rel);
                *v = V3::new(r.x + p.pivot.x, r.y + p.pivot.y, r.z + p.pivot.z);
            }
            q.normal = rot_mat.transform(q.normal);
            quads.push(q);
        }
    }
    quads
}
