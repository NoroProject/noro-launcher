//! Animation, back-face culling, lighting and projection into screen space.

use super::geometry::cuboid_quads;
use super::math::{Mat3, V3};
use super::model::BodyPartDef;
use super::part::BodyPartType;

/// Two soft key lights, mirrored around the vertical axis.
const LIGHT0: V3 = V3::new(
    0.161_690_416_690_888_66,
    0.808_452_083_454_443_2,
    -0.565_916_458_418_110_2,
);
const LIGHT1: V3 = V3::new(
    -0.161_690_416_690_888_66,
    0.808_452_083_454_443_2,
    0.565_916_458_418_110_2,
);

/// Peak swing of arms and legs.
const SWAY_ANGLE: f64 = 15.0;

pub struct ProjectedQuad {
    pub verts: [V3; 4],
    pub uvs: [(f64, f64); 4],
    pub avg_z: f64,
    pub allow_transparency: bool,
    pub shade: u8,
    pub kind: BodyPartType,
}

/// Swing the part, drop its hidden faces and project the rest into screen space.
/// `sway` is the loop progress in `[0, 1)`.
pub fn project_part(def: &BodyPartDef, rot: &Mat3, sway: f64, out: &mut Vec<ProjectedQuad>) {
    let mut quads = cuboid_quads(def);

    let phase = sway * std::f64::consts::TAU * def.kind.sway_time_mult();
    let pitch =
        -SWAY_ANGLE.to_radians() * def.kind.sway_strength() * phase.sin() + def.kind.pitch_offset();
    if pitch != 0.0 {
        let swing = Mat3::rotation_x(pitch);
        let pivot = def.pivot_or_centre();
        for quad in &mut quads {
            for vert in &mut quad.verts {
                *vert = swing.transform_around(*vert, pivot);
            }
            quad.normal = swing.transform(quad.normal);
        }
    }

    for quad in &quads {
        let mut normal = rot.transform(quad.normal);
        if normal.z <= 0.0 {
            if !quad.allow_transparency {
                continue; // back face of an opaque box — never visible
            }
            // Transparent layers stay visible from inside; light them as if facing us.
            normal = normal.negated();
        }

        let lit =
            (normal.dot(LIGHT0).clamp(0.0, 1.0) + normal.dot(LIGHT1).clamp(0.0, 1.0)).min(1.0);
        let shade = ((lit * 0.4 + 0.6).clamp(0.0, 1.0) * 255.0) as u8;

        let mut verts = [V3::new(0.0, 0.0, 0.0); 4];
        let mut avg_z = 0.0;
        for (i, v) in quad.verts.iter().enumerate() {
            let mut rv = rot.transform(*v);
            rv.y *= -1.0; // screen space grows downwards
            verts[i] = rv;
            avg_z += rv.z;
        }

        out.push(ProjectedQuad {
            verts,
            uvs: quad.uvs,
            avg_z: avg_z / 4.0,
            allow_transparency: quad.allow_transparency,
            shade,
            kind: def.kind,
        });
    }
}
