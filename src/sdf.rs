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

	fn to_floats(&self) -> Vec<Scalar> {
		match self {
			SDF::Sphere(r) => vec![0.0, *r],
			SDF::Translation(t, a) => vec![vec![1.0, t.0, t.1, t.2], a.to_floats()].concat(),
			SDF::Union(a, b) => vec![a.to_floats(), b.to_floats(), vec![2.0]].concat(),
			SDF::Intersection(a, b) => vec![a.to_floats(), b.to_floats(), vec![3.0]].concat(),
			SDF::Subtraction(a, b) => vec![a.to_floats(), b.to_floats(), vec![4.0]].concat(),
		}
	}

	pub fn to_bytes(&self) -> Vec<u8> {
		self.to_floats().into_iter().map(Scalar::to_le_bytes).flatten().collect()
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
