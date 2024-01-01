use std::ops::*;

pub use std::f32::consts::PI;

/// A vector in R^3.
#[derive(Clone, Copy, PartialEq)]
#[repr(align(16))] // WGSL `vec3`s are 16-byte aligned
#[must_use]
pub struct Vec3 {
    /// The x-coordinate.
    pub x: f32,
    /// The y-coordinate.
    pub y: f32,
    /// The z-coordinate.
    pub z: f32,
}

impl Vec3 {
    /// Apply an element-wise unary operation to a vector.
    /// This had better be inlined.
    pub fn unary(self, f: impl Fn(f32) -> f32) -> Vec3 {
        Vec3 {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
        }
    }

    /// Apply an element-wise binary operation to two vectors.
    /// This had better be inlined.
    pub fn binary(self, other: Vec3, f: impl Fn(f32, f32) -> f32) -> Vec3 {
        Vec3 {
            x: f(self.x, other.x),
            y: f(self.y, other.y),
            z: f(self.z, other.z),
        }
    }

    /// Reduce the elements of a vector using a binary operation.
    /// This had better be inlined.
    pub fn reduce(self, f: impl Fn(f32, f32) -> f32) -> f32 {
        f(self.x, f(self.y, self.z))
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        self.binary(other, f32::add)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        self.binary(other, f32::sub)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        self.unary(f32::neg)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, other: f32) -> Vec3 {
        self.unary(|x| x * other)
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, other: f32) -> Vec3 {
        self.unary(|x| x / other)
    }
}

impl Vec3 {
    /// Get the dot product of this vector and another one.
    pub fn dot(self, other: Vec3) -> f32 {
        self.binary(other, f32::mul).reduce(f32::add)
    }

    /// Get the length of this vector.
    #[must_use]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Get a vector in the same direction as this one, but with unit length.
    pub fn normalized(self) -> Vec3 {
        self / self.length()
    }
}

impl std::fmt::Debug for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:?}, {:?}, {:?})", self.x, self.y, self.z)
    }
}

/// The origin, a.k.a. the zero vector.
pub const ORIGIN: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

/// The unit vector in the x direction.
pub const X: Vec3 = Vec3 {
    x: 1.0,
    y: 0.0,
    z: 0.0,
};

/// The unit vector in the y direction.
pub const Y: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

/// The unit vector in the z direction.
pub const Z: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 1.0,
};
