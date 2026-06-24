//! Geometric math primitives for the CAD kernel.
//!
//! Phase 0 foundation. Right now this is just a 3D vector type with the
//! operations the topology + tessellation code needs. It will grow into the
//! home for transforms, tolerance handling, and the analytic curve/surface
//! math as the kernel develops.

use std::ops::{Add, Div, Mul, Neg, Sub};

/// A 3D vector. We currently also use this to represent points in space.
///
/// Eventually it is likely worth giving points their own distinct type, since
/// "point - point = vector" but "point + point" is meaningless. For now the
/// single type keeps the early code light.
pub type Point3 = Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    pub fn dot(self, rhs: Vec3) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Returns the unit vector in the same direction.
    ///
    /// Returns `ZERO` for a zero-length input rather than producing NaNs; the
    /// caller is responsible for not relying on the direction in that case.
    pub fn normalized(self) -> Vec3 {
        let len = self.length();
        if len == 0.0 {
            Vec3::ZERO
        } else {
            self / len
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, s: f64) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, s: f64) -> Vec3 {
        Vec3::new(self.x / s, self.y / s, self.z / s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cross_of_basis_vectors() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(x.cross(y), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn normalized_unit_length() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert!((v.length() - 5.0).abs() < 1e-12);
        assert!((v.normalized().length() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn normalized_zero_is_zero() {
        assert_eq!(Vec3::ZERO.normalized(), Vec3::ZERO);
    }
}
