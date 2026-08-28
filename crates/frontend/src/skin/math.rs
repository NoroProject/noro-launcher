//! Vector / matrix math for the skin renderer.

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

    pub fn dot(self, other: V3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn negated(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

/// 3×3 rotation matrix stored as rows.
pub struct Mat3([[f64; 3]; 3]);

impl Mat3 {
    /// Yaw around Y, then pitch around X.
    pub fn rotation_yx(ry: f64, rx: f64) -> Self {
        let (sy, cy) = ry.sin_cos();
        let (sx, cx) = rx.sin_cos();
        Self([
            [cy, 0.0, sy],
            [sx * sy, cx, -sx * cy],
            [-cx * sy, sx, cx * cy],
        ])
    }

    pub fn rotation_x(rx: f64) -> Self {
        let (sx, cx) = rx.sin_cos();
        Self([[1.0, 0.0, 0.0], [0.0, cx, -sx], [0.0, sx, cx]])
    }

    pub fn transform(&self, v: V3) -> V3 {
        let r = &self.0;
        V3 {
            x: r[0][0] * v.x + r[0][1] * v.y + r[0][2] * v.z,
            y: r[1][0] * v.x + r[1][1] * v.y + r[1][2] * v.z,
            z: r[2][0] * v.x + r[2][1] * v.y + r[2][2] * v.z,
        }
    }

    /// Rotate `v` around `pivot` instead of the origin.
    pub fn transform_around(&self, v: V3, pivot: V3) -> V3 {
        let local = V3::new(v.x - pivot.x, v.y - pivot.y, v.z - pivot.z);
        let r = self.transform(local);
        V3::new(r.x + pivot.x, r.y + pivot.y, r.z + pivot.z)
    }
}
