//! Unwrapping a body part box into six textured quads.

use super::math::V3;
use super::model::BodyPartDef;
use super::part::BodyPartType;

pub struct Quad {
    pub verts: [V3; 4],
    /// Absolute texture coordinates, in 64×64 skin pixels.
    pub uvs: [(f64, f64); 4],
    pub normal: V3,
    pub allow_transparency: bool,
}

impl Quad {
    fn flip_horz(mut self, flip: bool) -> Self {
        if flip {
            self.uvs.swap(0, 1);
            self.uvs.swap(2, 3);
        }
        self
    }

    fn flip_vert(mut self, flip: bool) -> Self {
        if flip {
            self.uvs.swap(0, 3);
            self.uvs.swap(1, 2);
        }
        self
    }
}

/// Six faces of the part's box, in the standard Minecraft skin layout.
pub fn cuboid_quads(def: &BodyPartDef) -> [Quad; 6] {
    let inflate = def.kind.inflate();
    let transparent = def.kind.allow_transparency();
    let (w, h, d) = (
        def.max.x - def.min.x,
        def.max.y - def.min.y,
        def.max.z - def.min.z,
    );
    let (x0, y0, z0) = (
        def.min.x - inflate,
        def.min.y - inflate,
        def.min.z - inflate,
    );
    let (x1, y1, z1) = (
        def.max.x + inflate,
        def.max.y + inflate,
        def.max.z + inflate,
    );
    let (tx, ty) = (def.tx, def.ty);
    let flip_x = def.flip_x;
    // The cape texture is seen from the back, so its sides and caps are mirrored.
    let flip_z = def.kind == BodyPartType::Cape;

    let quad = |verts: [V3; 4], uvs: [(f64, f64); 4], normal: V3| Quad {
        verts,
        uvs,
        normal,
        allow_transparency: transparent,
    };

    let mut quads = [
        // Front (+Z)
        quad(
            [
                V3::new(x0, y1, z1),
                V3::new(x1, y1, z1),
                V3::new(x1, y0, z1),
                V3::new(x0, y0, z1),
            ],
            [
                (tx + d, ty + d),
                (tx + d + w, ty + d),
                (tx + d + w, ty + d + h),
                (tx + d, ty + d + h),
            ],
            V3::new(0.0, 0.0, 1.0),
        )
        .flip_horz(flip_x),
        // Back (-Z)
        quad(
            [
                V3::new(x1, y1, z0),
                V3::new(x0, y1, z0),
                V3::new(x0, y0, z0),
                V3::new(x1, y0, z0),
            ],
            [
                (tx + 2.0 * d + w, ty + d),
                (tx + 2.0 * d + 2.0 * w, ty + d),
                (tx + 2.0 * d + 2.0 * w, ty + d + h),
                (tx + 2.0 * d + w, ty + d + h),
            ],
            V3::new(0.0, 0.0, -1.0),
        )
        .flip_horz(flip_x),
        // Right (-X, the player's right)
        quad(
            [
                V3::new(x0, y1, z1),
                V3::new(x0, y1, z0),
                V3::new(x0, y0, z0),
                V3::new(x0, y0, z1),
            ],
            [
                (tx + d, ty + d),
                (tx, ty + d),
                (tx, ty + d + h),
                (tx + d, ty + d + h),
            ],
            V3::new(-1.0, 0.0, 0.0),
        )
        .flip_horz(flip_z),
        // Left (+X)
        quad(
            [
                V3::new(x1, y1, z0),
                V3::new(x1, y1, z1),
                V3::new(x1, y0, z1),
                V3::new(x1, y0, z0),
            ],
            [
                (tx + 2.0 * d + w, ty + d),
                (tx + d + w, ty + d),
                (tx + d + w, ty + d + h),
                (tx + 2.0 * d + w, ty + d + h),
            ],
            V3::new(1.0, 0.0, 0.0),
        )
        .flip_horz(flip_z),
        // Top (+Y)
        quad(
            [
                V3::new(x0, y1, z0),
                V3::new(x1, y1, z0),
                V3::new(x1, y1, z1),
                V3::new(x0, y1, z1),
            ],
            [
                (tx + d, ty),
                (tx + d + w, ty),
                (tx + d + w, ty + d),
                (tx + d, ty + d),
            ],
            V3::new(0.0, 1.0, 0.0),
        )
        .flip_horz(flip_x)
        .flip_vert(flip_z),
        // Bottom (-Y)
        quad(
            [
                V3::new(x1, y0, z0),
                V3::new(x0, y0, z0),
                V3::new(x0, y0, z1),
                V3::new(x1, y0, z1),
            ],
            [
                (tx + d + 2.0 * w, ty),
                (tx + d + w, ty),
                (tx + d + w, ty + d),
                (tx + d + 2.0 * w, ty + d),
            ],
            V3::new(0.0, -1.0, 0.0),
        )
        .flip_horz(flip_x)
        .flip_vert(flip_z),
    ];

    // Mirrored parts also swap which side face samples which texture region.
    if flip_x {
        swap_uvs(&mut quads, 2, 3);
    }
    if flip_z {
        swap_uvs(&mut quads, 0, 1);
    }
    quads
}

fn swap_uvs(quads: &mut [Quad; 6], a: usize, b: usize) {
    let uvs = quads[a].uvs;
    quads[a].uvs = quads[b].uvs;
    quads[b].uvs = uvs;
}
