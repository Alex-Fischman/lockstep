use std::ops::*;

pub type Scalar = f32;

#[derive(Clone, Copy, Debug)]
pub struct Vector(pub Scalar, pub Scalar, pub Scalar);

impl Add for Vector {
	type Output = Vector;
	fn add(self, other: Vector) -> Vector {
		Vector(self.0 + other.0, self.1 + other.1, self.2 + other.2)
	}
}

impl Mul<Vector> for Scalar {
	type Output = Vector;
	fn mul(self, other: Vector) -> Vector {
		Vector(self * other.0, self * other.1, self * other.2)
	}
}

impl BitOr for Vector {
	type Output = Scalar;
	fn bitor(self, other: Vector) -> Scalar {
		self.0 * other.0 + self.1 * other.1 + self.2 * other.2
	}
}

impl Vector {
	pub fn length(self) -> Scalar {
		(self | self).sqrt()
	}

	pub fn normalized(self) -> Option<Vector> {
		let l = 1.0 / self.length();
		if l.is_nan() {
			None
		} else {
			Some(l * self)
		}
	}
}

pub fn sphere(v: Vector) -> Scalar {
	return v.length() - 1.0;
}

pub struct Ray {
	pub pos: Vector,
	pub dir: Vector,
	pub min_dist: Scalar,
	pub sum_dist: Scalar,
	pub iterations: usize,
}

const MIN_DIST: Scalar = 0.01;
const MAX_DIST: Scalar = 1000.0;
const MAX_ITER: usize = 20;
pub fn raymarch<SDF: Fn(Vector) -> Scalar>(pos: Vector, dir: Vector, sdf: SDF) -> Ray {
	let mut ray = Ray { pos, dir, min_dist: Scalar::MAX, sum_dist: 0.0, iterations: 0 };
	ray.dir = ray.dir.normalized().unwrap();
	loop {
		let dist = sdf(ray.pos);
		if dist < MIN_DIST || dist > MAX_DIST || ray.iterations > MAX_ITER {
			return ray;
		}
		ray.pos = ray.pos + dist * dir;
		ray.min_dist = ray.min_dist.min(dist);
		ray.sum_dist += dist;
		ray.iterations += 1;
	}
}