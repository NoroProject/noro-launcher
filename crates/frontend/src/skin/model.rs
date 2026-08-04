//! Player model tables: classic (4px arms), slim (3px arms) and legacy 64×32 skins.

use super::math::V3;
use super::part::BodyPartType::{self, *};

pub struct BodyPartDef {
    pub min: V3,
    pub max: V3,
    /// Rotation pivot for the swing; defaults to the box centre.
    pub pivot: Option<V3>,
    /// Texture origin of the unwrapped box, in 64×64 skin pixels.
    pub tx: f64,
    pub ty: f64,
    /// Legacy skins mirror the right limb onto the left one.
    pub flip_x: bool,
    pub kind: BodyPartType,
}

impl BodyPartDef {
    fn flipped(mut self) -> Self {
        self.flip_x = true;
        self
    }

    pub fn pivot_or_centre(&self) -> V3 {
        self.pivot.unwrap_or_else(|| {
            V3::new(
                (self.min.x + self.max.x) / 2.0,
                (self.min.y + self.max.y) / 2.0,
                (self.min.z + self.max.z) / 2.0,
            )
        })
    }
}

fn part(min: V3, max: V3, pivot: Option<V3>, tx: f64, ty: f64, kind: BodyPartType) -> BodyPartDef {
    BodyPartDef {
        min,
        max,
        pivot,
        tx,
        ty,
        flip_x: false,
        kind,
    }
}

/// All parts of the player, ordered base layer first, overlays last.
pub fn player_model(is_legacy: bool, is_slim: bool) -> Vec<BodyPartDef> {
    let aw = if is_slim { 3.0 } else { 4.0 };
    let head = |tx, ty, kind| {
        part(
            V3::new(-4.0, 8.0, -4.0),
            V3::new(4.0, 16.0, 4.0),
            None,
            tx,
            ty,
            kind,
        )
    };
    let body = |tx, ty, kind| {
        part(
            V3::new(-4.0, -4.0, -2.0),
            V3::new(4.0, 8.0, 2.0),
            None,
            tx,
            ty,
            kind,
        )
    };
    let pivot_r_arm = Some(V3::new(-4.0, 8.0, 0.0));
    let pivot_l_arm = Some(V3::new(4.0, 8.0, 0.0));
    let r_arm = |tx, ty, kind| {
        part(
            V3::new(-4.0 - aw, -4.0, -2.0),
            V3::new(-4.0, 8.0, 2.0),
            pivot_r_arm,
            tx,
            ty,
            kind,
        )
    };
    let l_arm = |tx, ty, kind| {
        part(
            V3::new(4.0, -4.0, -2.0),
            V3::new(4.0 + aw, 8.0, 2.0),
            pivot_l_arm,
            tx,
            ty,
            kind,
        )
    };
    let pivot_r_leg = Some(V3::new(-2.0, -4.0, 0.0));
    let pivot_l_leg = Some(V3::new(2.0, -4.0, 0.0));
    let r_leg = |tx, ty, kind| {
        part(
            V3::new(-4.0, -16.0, -2.0),
            V3::new(0.0, -4.0, 2.0),
            pivot_r_leg,
            tx,
            ty,
            kind,
        )
    };
    let l_leg = |tx, ty, kind| {
        part(
            V3::new(0.0, -16.0, -2.0),
            V3::new(4.0, -4.0, 2.0),
            pivot_l_leg,
            tx,
            ty,
            kind,
        )
    };

    if is_legacy {
        return vec![
            head(0.0, 0.0, Head),
            body(16.0, 16.0, Body),
            r_arm(40.0, 16.0, RightArm),
            l_arm(40.0, 16.0, LeftArm).flipped(),
            r_leg(0.0, 16.0, RightLeg),
            l_leg(0.0, 16.0, LeftLeg).flipped(),
            head(32.0, 0.0, HeadOverlay),
        ];
    }

    vec![
        head(0.0, 0.0, Head),
        body(16.0, 16.0, Body),
        r_arm(40.0, 16.0, RightArm),
        l_arm(32.0, 48.0, LeftArm),
        r_leg(0.0, 16.0, RightLeg),
        l_leg(16.0, 48.0, LeftLeg),
        head(32.0, 0.0, HeadOverlay),
        body(16.0, 32.0, BodyOverlay),
        r_arm(40.0, 32.0, RightArmOverlay),
        l_arm(48.0, 48.0, LeftArmOverlay),
        r_leg(0.0, 32.0, RightLegOverlay),
        l_leg(0.0, 48.0, LeftLegOverlay),
    ]
}

/// Cape box, sampled from the cape texture instead of the skin.
pub fn cape_part() -> BodyPartDef {
    part(
        V3::new(-5.0, -8.0, -3.0),
        V3::new(5.0, 8.0, -2.0),
        Some(V3::new(0.0, 8.0, -2.0)),
        0.0,
        0.0,
        Cape,
    )
}
