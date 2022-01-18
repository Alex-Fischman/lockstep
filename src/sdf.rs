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

pub enum SDF {
	Sphere(Scalar),
	Translation(Vector, Box<SDF>),
	Union(Box<SDF>, Box<SDF>),
	Intersection(Box<SDF>, Box<SDF>),
	Subtraction(Box<SDF>, Box<SDF>),
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

#[allow(dead_code)]
impl SDF {
	pub fn sphere(r: Scalar) -> SDF {
		SDF::Sphere(r)
	}

	pub fn translate(self, t: Vector) -> SDF {
		SDF::Translation(t, Box::new(self))
	}

	pub fn unite(self, other: SDF) -> SDF {
		SDF::Union(Box::new(self), Box::new(other))
	}

	pub fn intersect(self, other: SDF) -> SDF {
		SDF::Intersection(Box::new(self), Box::new(other))
	}

	pub fn subtract(self, other: SDF) -> SDF {
		SDF::Subtraction(Box::new(self), Box::new(other))
	}

	fn to_floats(&self) -> Vec<f32> {
		match self {
			SDF::Sphere(r) => vec![0.0, *r],
			SDF::Translation(t, a) => {
				let mut floats = vec![1.0, t.0, t.1, t.2];
				floats.extend(a.to_floats());
				floats
			}
			SDF::Union(a, b) => {
				let mut floats = vec![];
				floats.extend(a.to_floats());
				floats.extend(b.to_floats());
				floats.push(2.0);
				floats
			}
			SDF::Intersection(a, b) => {
				let mut floats = vec![];
				floats.extend(a.to_floats());
				floats.extend(b.to_floats());
				floats.push(3.0);
				floats
			}
			SDF::Subtraction(a, b) => {
				let mut floats = vec![];
				floats.extend(a.to_floats());
				floats.extend(b.to_floats());
				floats.push(4.0);
				floats
			}
		}
	}

	pub fn to_bytes(&self) -> &[u8] {
		let floats = self.to_floats();
		unsafe { std::slice::from_raw_parts(floats.as_ptr() as *const u8, floats.len() * 4) }
	}

	fn distance(&self, v: Vector) -> Scalar {
		match self {
			SDF::Sphere(r) => v.length() - r,
			SDF::Translation(t, a) => a.distance(v + -1.0 * *t),
			SDF::Union(a, b) => a.distance(v).min(b.distance(v)),
			SDF::Intersection(a, b) => a.distance(v).max(b.distance(v)),
			SDF::Subtraction(a, b) => (-a.distance(v)).max(b.distance(v)),
		}
	}

	pub fn raymarch(&self, pos: Vector, dir: Vector) -> Ray {
		let mut ray = Ray { pos, dir, min_dist: Scalar::MAX, sum_dist: 0.0, iterations: 0 };
		ray.dir = ray.dir.normalized().unwrap();
		loop {
			let dist = self.distance(ray.pos);
			if dist < MIN_DIST || dist > MAX_DIST || ray.iterations > MAX_ITER {
				return ray;
			}
			ray.pos = ray.pos + dist * dir;
			ray.min_dist = ray.min_dist.min(dist);
			ray.sum_dist += dist;
			ray.iterations += 1;
		}
	}
}
