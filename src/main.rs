use sdl2::pixels::Color;

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;
const PIXELS: usize = (WIDTH * HEIGHT) as usize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let sdl = sdl2::init()?;
	let mut pump = sdl.event_pump()?;
	let video = sdl.video()?;
	let window = video.window("lockstep", WIDTH, WIDTH).position_centered().build()?;
	let mut canvas = window.into_canvas().build()?;
	let texture_creator = canvas.texture_creator();
	let mut texture = texture_creator.create_texture_streaming(
		Some(sdl2::pixels::PixelFormatEnum::RGBA32),
		WIDTH,
		HEIGHT,
	)?;
	let mut pixels = [Color::BLACK; PIXELS];

	let then = std::time::Instant::now();
	render(&mut pixels);
	let now = std::time::Instant::now();
	println!("{:?}", now - then);

	texture.update(
		None,
		unsafe { std::slice::from_raw_parts(pixels.as_ptr() as *const u8, PIXELS) },
		std::mem::size_of::<Color>() * WIDTH as usize,
	)?;
	canvas.copy(&texture, None, None)?;
	canvas.present();

	loop {
		for event in pump.poll_iter() {
			match event {
				sdl2::event::Event::Quit { .. } => return Ok(()),
				_ => {}
			}
		}
	}
}

#[derive(Clone, Copy, Debug)]
struct Vector(f64, f64, f64);

impl std::ops::Add for Vector {
	type Output = Vector;
	fn add(self, other: Vector) -> Vector {
		Vector(self.0 + other.0, self.1 + other.1, self.2 + other.2)
	}
}

impl std::ops::Sub for Vector {
	type Output = Vector;
	fn sub(self, other: Vector) -> Vector {
		Vector(self.0 - other.0, self.1 - other.1, self.2 - other.2)
	}
}

impl std::ops::Neg for Vector {
	type Output = Vector;
	fn neg(self) -> Vector {
		Vector(-self.0, -self.1, -self.2)
	}
}

impl std::ops::Mul<Vector> for f64 {
	type Output = Vector;
	fn mul(self, other: Vector) -> Vector {
		Vector(self * other.0, self * other.1, self * other.2)
	}
}

#[allow(dead_code)]
impl Vector {
	const ZERO: Vector = Vector(0.0, 0.0, 0.0);
	const X: Vector = Vector(1.0, 0.0, 0.0);
	const Y: Vector = Vector(0.0, 1.0, 0.0);
	const Z: Vector = Vector(0.0, 0.0, 1.0);

	fn length(self) -> f64 {
		(self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt()
	}

	fn normalized(self) -> Vector {
		1.0 / self.length() * self
	}
}

#[derive(Clone, Copy, Debug)]
struct Ray {
	pos: Vector,
	dir: Vector,
}

impl Ray {
	fn new(pos: Vector, dir: Vector) -> Ray {
		Ray { pos, dir: dir.normalized() }
	}
}

#[derive(Debug)]
enum SDF<'a> {
	Sphere(f64),
	Union(&'a SDF<'a>, &'a SDF<'a>),
	Translate(&'a SDF<'a>, Vector),
	Extrude(&'a SDF<'a>, Vector),
}

impl<'a> SDF<'a> {
	const ITERATION_MAX: u64 = 100;
	fn run(&self, v: Vector) -> f64 {
		match self {
			SDF::Sphere(r) => v.length() - r,
			SDF::Union(a, b) => f64::min(a.run(v), b.run(v)),
			SDF::Translate(a, u) => a.run(v - *u),
			SDF::Extrude(a, u) => {
				let ray = Ray::new(v, -*u);
				let mut min_dist = f64::MAX;
				let mut distance = 0.0;
				let mut iteration = 0;
				while distance < u.length() && iteration < SDF::ITERATION_MAX {
					let d = a.run(ray.pos + distance * ray.dir);
					min_dist = min_dist.min(d);
					distance += d.abs();
					iteration += 1;
				}
				min_dist
			}
		}
	}

	fn union(&'a self, other: &'a SDF) -> SDF<'a> {
		SDF::Union(self, other)
	}

	fn translate(&'a self, t: Vector) -> SDF<'a> {
		SDF::Translate(self, t)
	}

	const DISTANCE_MIN: f64 = 0.01;
	const DISTANCE_MAX: f64 = 10.0;
	fn raymarch(&self, ray: Ray) -> Option<(f64, u64)> {
		let mut distance = 0.0;
		let mut iteration = 0;
		while distance < SDF::DISTANCE_MAX && iteration < SDF::ITERATION_MAX {
			match self.run(ray.pos + distance * ray.dir) {
				d if d < SDF::DISTANCE_MIN => return Some((distance, iteration)),
				d => distance += d,
			}
			iteration += 1;
		}
		None
	}
}

use std::f64::consts::PI;
const FOV: f64 = PI / 2.0;
fn render(pixels: &mut [Color; PIXELS]) {
	let a = SDF::Sphere(1.00).translate(Vector(0.5, 0.0, 2.0));
	let b = SDF::Sphere(0.75).translate(Vector(-1.0, 0.0, 3.0));
	let c = SDF::union(&a, &b);
	let scene = SDF::Extrude(&c, -Vector::Y);
	for x in 0..WIDTH {
		for y in 0..HEIGHT {
			let i = (x + y * HEIGHT) as usize;
			let x = x as f64 / WIDTH as f64 - 0.5;
			let y = y as f64 / HEIGHT as f64 - 0.5;
			let z = (PI * 0.5 - FOV * 0.5).tan() * 0.5;
			let ray = Ray::new(Vector::ZERO, Vector(x, y, z).normalized());
			pixels[i] = match scene.raymarch(ray) {
				None => Color::BLACK,
				Some((_dist, _iter)) => Color::WHITE,
			}
		}
	}
}
