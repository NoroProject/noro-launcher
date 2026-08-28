//! Body part kinds and their per-kind rendering behaviour.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BodyPartType {
    Head,
    Body,
    RightArm,
    LeftArm,
    RightLeg,
    LeftLeg,
    HeadOverlay,
    BodyOverlay,
    RightArmOverlay,
    LeftArmOverlay,
    RightLegOverlay,
    LeftLegOverlay,
    Cape,
}

use BodyPartType::*;

impl BodyPartType {
    /// How much the box grows outwards, so overlays don't z-fight with the base layer.
    pub fn inflate(self) -> f64 {
        match self {
            HeadOverlay | RightArmOverlay | LeftArmOverlay | RightLegOverlay | LeftLegOverlay => {
                0.25
            }
            BodyOverlay => 0.24,
            _ => 0.0,
        }
    }

    /// Overlay layers keep their alpha; base layers are drawn opaque.
    pub fn allow_transparency(self) -> bool {
        matches!(
            self,
            HeadOverlay
                | BodyOverlay
                | RightArmOverlay
                | LeftArmOverlay
                | RightLegOverlay
                | LeftLegOverlay
        )
    }

    /// Limbs swing four times per loop, the cape sways once.
    pub fn sway_time_mult(self) -> f64 {
        if self == Cape {
            1.0
        } else {
            4.0
        }
    }

    /// Sign and amount of the swing; opposite limbs move in antiphase.
    pub fn sway_strength(self) -> f64 {
        match self {
            RightArm | RightArmOverlay | LeftLeg | LeftLegOverlay => 1.0,
            LeftArm | LeftArmOverlay | RightLeg | RightLegOverlay => -1.0,
            Cape => 0.25,
            _ => 0.0,
        }
    }

    /// Resting angle — the cape hangs away from the back.
    pub fn pitch_offset(self) -> f64 {
        if self == Cape {
            18.75_f64.to_radians()
        } else {
            0.0
        }
    }
}
