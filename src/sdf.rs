use crate::*;

/// An `Sdf` represents a Constructive Solid Geometry DAG.
#[must_use]
pub struct Sdf {
    distances: Vec<Distance>,
    materials: Vec<Material>,
}

/// A node in an `Sdf` DAG.
// Invariant: `Distance`s should only point to `Distance`s that have a strictly smaller index.
// This both ensures that the graph is a DAG, and makes distance computations simpler.
#[derive(PartialEq)]
enum Distance {
    // primitives
    Sphere {
        center: Vec3,
        radius: f32,
        material: usize,
    },
    Plane {
        normal: Vec3, // must be normalized
        offset: f32,
        material: usize,
    },
    // operations
    Union(usize, usize),
    Intersection(usize, usize),
    Exclusion(usize, usize),
    Subtraction(usize, usize),
}

impl Sdf {
    fn simplify(self) -> Sdf {
        let mut materials = vec![];
        let mut material_map: HashMap<usize, usize> = HashMap::new();
        for (i, material) in self.materials.into_iter().enumerate() {
            if let Some(j) = materials.iter().position(|x| *x == material) {
                material_map.insert(i, j);
            } else {
                material_map.insert(i, materials.len());
                materials.push(material);
            }
        }

        let mut distances = vec![];
        let mut distance_map: HashMap<usize, usize> = HashMap::new();
        for (i, mut distance) in self.distances.into_iter().enumerate() {
            match &mut distance {
                Distance::Sphere { material, .. } | Distance::Plane { material, .. } => {
                    *material = material_map[material];
                }
                Distance::Union(x, y)
                | Distance::Intersection(x, y)
                | Distance::Exclusion(x, y)
                | Distance::Subtraction(x, y) => {
                    *x = distance_map[x];
                    *y = distance_map[y];
                }
            }

            if let Some(j) = distances.iter().position(|x| *x == distance) {
                distance_map.insert(i, j);
            } else {
                distance_map.insert(i, distances.len());
                distances.push(distance);
            }
        }

        Sdf {
            distances,
            materials,
        }
    }

    fn append(mut self, mut other: Sdf) -> Sdf {
        let distance_offset = self.distances.len();
        let material_offset = self.materials.len();

        self.materials.append(&mut other.materials);

        for mut distance in other.distances {
            match &mut distance {
                Distance::Sphere { material, .. } | Distance::Plane { material, .. } => {
                    *material += material_offset;
                }
                Distance::Union(x, y)
                | Distance::Intersection(x, y)
                | Distance::Exclusion(x, y)
                | Distance::Subtraction(x, y) => {
                    *x += distance_offset;
                    *y += distance_offset;
                }
            }
            self.distances.push(distance);
        }

        self
    }

    /// Construct a new SDF of a sphere, centered at the origin.
    pub fn sphere(radius: f32, material: Material) -> Sdf {
        Sdf {
            distances: vec![Distance::Sphere {
                center: ORIGIN,
                radius,
                material: 0,
            }],
            materials: vec![material],
        }
    }

    /// Construct a new SDF of a cube, centered at the origin.
    pub fn plane(normal: Vec3, offset: f32, material: Material) -> Sdf {
        Sdf {
            distances: vec![Distance::Plane {
                normal,
                offset,
                material: 0,
            }],
            materials: vec![material],
        }
    }

    /// Union another SDF into this one.
    pub fn union(self, other: Sdf) -> Sdf {
        let self_root = self.distances.len() - 1;
        let mut out = self.append(other);
        out.distances
            .push(Distance::Union(self_root, out.distances.len() - 1));
        out.simplify()
    }

    /// Intersect another SDF with this one.
    pub fn intersect(self, other: Sdf) -> Sdf {
        let self_root = self.distances.len() - 1;
        let mut out = self.append(other);
        out.distances
            .push(Distance::Intersection(self_root, out.distances.len() - 1));
        out.simplify()
    }

    /// Construct an SDF that contains the points in either this SDF XOP the other SDF.
    pub fn exclude(self, other: Sdf) -> Sdf {
        let self_root = self.distances.len() - 1;
        let mut out = self.append(other);
        out.distances
            .push(Distance::Exclusion(self_root, out.distances.len() - 1));
        out.simplify()
    }

    /// Subtract another SDF from this one.
    pub fn subtract(self, other: Sdf) -> Sdf {
        let self_root = self.distances.len() - 1;
        let mut out = self.append(other);
        out.distances
            .push(Distance::Subtraction(self_root, out.distances.len() - 1));
        out.simplify()
    }

    /// Translate this SDF along a vector.
    pub fn translate(mut self, vec: Vec3) -> Sdf {
        for distance in &mut self.distances {
            match distance {
                Distance::Sphere { center, .. } => {
                    *center = *center + vec;
                }
                Distance::Plane { normal, offset, .. } => {
                    *offset += vec.dot(*normal);
                }
                Distance::Union(..)
                | Distance::Intersection(..)
                | Distance::Exclusion(..)
                | Distance::Subtraction(..) => {}
            }
        }
        self
    }
}

/// Represents the way that an object's pixels are colored.
#[derive(Debug, PartialEq)]
pub enum Material {
    /// A basic material that sets all the pixels of an object to the same color.
    Flat(Color),
}

/// A struct to hold the information returned from a raymarching call.
#[must_use]
pub enum Raymarch {
    /// The ray collided with an object.
    Hit(Vec3),
    /// The ray exceeded the `MAX_DIST` rendering limit.
    WentTooFar,
    /// The ray did not collide with an object before `MAX_ITER` iterations.
    TookTooLong,
}

/// Epsilon for floating point equality check in raymarching
pub const MIN_DIST: f32 = 0.01;
/// Ray length cutoff for raymarching, similar to a "far plane"
pub const MAX_DIST: f32 = 10.0;
/// Maximum iteration count for raymarching
pub const MAX_ITER: usize = 20;

impl Sdf {
    /// Get the signed distance to this `Sdf` from some point `p`.
    #[must_use]
    pub fn distance(&self, p: Vec3) -> f32 {
        let mut distances = vec![0.0; self.distances.len()];
        for (i, node) in self.distances.iter().enumerate() {
            distances[i] = match node {
                Distance::Sphere { center, radius, .. } => (p - *center).length() - radius,
                Distance::Plane { normal, offset, .. } => p.dot(*normal) - offset,
                Distance::Union(x, y) => distances[*x].min(distances[*y]),
                Distance::Intersection(x, y) => distances[*x].max(distances[*y]),
                Distance::Exclusion(x, y) => distances[*x]
                    .min(distances[*y])
                    .max(-distances[*x].max(distances[*y])),
                Distance::Subtraction(x, y) => (-distances[*x]).max(distances[*y]),
            };
        }
        distances.pop().unwrap()
    }

    /// Find where the given ray collides with this SDF, if it does.
    pub fn raymarch(&self, pos: Vec3, mut dir: Vec3) -> Raymarch {
        let mut steps = 0;
        let mut accum = 0.0;
        dir = dir.normalized();
        while steps < MAX_ITER {
            let point = pos + dir * accum;
            let distance = self.distance(point);

            if distance < MIN_DIST {
                return Raymarch::Hit(point);
            } else if distance > MAX_DIST {
                return Raymarch::WentTooFar;
            }
            steps += 1;
            accum += distance;
        }
        Raymarch::TookTooLong
    }
}

/// A representation of the `Distance` enum that matches the struct in the shader.
#[repr(C)]
pub struct GpuDistance {
    tag: u32,
    x: u32,
    y: u32,
    _padding: u32,
    v: [f32; 4],
}

/// A representation of the `Material` enum that matches the struct in the shader.
#[repr(C)]
pub struct GpuMaterial {
    tag: u32,
    r: f32,
    g: f32,
    b: f32,
}

const MAGIC_U32: u32 = 0xDEAD_BEEF;
const MAGIC_F32: f32 = -12.34;

impl Sdf {
    /// Convert this `SDF` to a representation that can be sent to the shader.
    #[must_use]
    pub fn to_gpu_repr(&self) -> (Vec<GpuDistance>, Vec<GpuMaterial>) {
        let distances: Vec<_> = self
            .distances
            .iter()
            .map(|distance| match distance {
                Distance::Sphere {
                    center,
                    radius,
                    material,
                } => GpuDistance {
                    tag: 0,
                    x: *material as u32,
                    y: MAGIC_U32,
                    _padding: MAGIC_U32,
                    v: [center.x, center.y, center.z, *radius],
                },
                Distance::Plane {
                    normal,
                    offset,
                    material,
                } => GpuDistance {
                    tag: 1,
                    x: *material as u32,
                    y: MAGIC_U32,
                    _padding: MAGIC_U32,
                    v: [normal.x, normal.y, normal.z, *offset],
                },
                Distance::Union(x, y) => GpuDistance {
                    tag: 2,
                    x: *x as u32,
                    y: *y as u32,
                    _padding: MAGIC_U32,
                    v: [MAGIC_F32, MAGIC_F32, MAGIC_F32, MAGIC_F32],
                },
                Distance::Intersection(x, y) => GpuDistance {
                    tag: 3,
                    x: *x as u32,
                    y: *y as u32,
                    _padding: MAGIC_U32,
                    v: [MAGIC_F32, MAGIC_F32, MAGIC_F32, MAGIC_F32],
                },
                Distance::Exclusion(x, y) => GpuDistance {
                    tag: 4,
                    x: *x as u32,
                    y: *y as u32,
                    _padding: MAGIC_U32,
                    v: [MAGIC_F32, MAGIC_F32, MAGIC_F32, MAGIC_F32],
                },
                Distance::Subtraction(x, y) => GpuDistance {
                    tag: 5,
                    x: *x as u32,
                    y: *y as u32,
                    _padding: MAGIC_U32,
                    v: [MAGIC_F32, MAGIC_F32, MAGIC_F32, MAGIC_F32],
                },
            })
            .collect();
        let materials: Vec<_> = self
            .materials
            .iter()
            .map(|material| match material {
                Material::Flat(color) => GpuMaterial {
                    tag: 0,
                    r: color.r as f32,
                    g: color.g as f32,
                    b: color.b as f32,
                },
            })
            .collect();
        (distances, materials)
    }
}
