use sdl2::pixels::Color;

const WIDTH: u32 = 500;
const HEIGHT: u32 = 500;
const PIXELS: usize = (WIDTH * HEIGHT) as usize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let sdl = sdl2::init()?;
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
	let mut then = std::time::Instant::now();
	let mut state = State::new();
	let mut pump = sdl.event_pump()?;
	loop {
		for event in pump.poll_iter() {
			match event {
				sdl2::event::Event::Quit { .. } => return Ok(()),
				_ => {}
			}
		}

		update(&mut state);
		render(&state, &mut pixels);
		texture.update(
			None,
			unsafe { std::slice::from_raw_parts(pixels.as_ptr() as *const u8, PIXELS) },
			std::mem::size_of::<Color>() * WIDTH as usize,
		)?;
		canvas.copy(&texture, None, None)?;
		canvas.present();

		let now = std::time::Instant::now();
		let delta = now - then;
		then = now;
		println!("{:?}", delta);
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

struct SDF(Box<dyn Fn(Vector) -> f64>);

impl std::fmt::Debug for SDF {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "SDF")
	}
}

impl SDF {
	fn sphere() -> SDF {
		SDF(Box::new(move |v: Vector| v.length() - 1.0))
	}

	fn union(a: SDF, b: SDF) -> SDF {
		SDF(Box::new(move |v: Vector| f64::min(a.0(v), b.0(v))))
	}

	fn translate(self, t: Vector) -> SDF {
		SDF(Box::new(move |v: Vector| self.0(v - t)))
	}

	fn scale(self, s: f64) -> SDF {
		SDF(Box::new(move |v: Vector| self.0(1.0 / s * v) * s))
	}
}

const DISTANCE_MIN: f64 = 0.01;
const DISTANCE_MAX: f64 = 10.0;
const ITERATION_MAX: u64 = 100;
fn raymarch(sdf: &SDF, ray: Ray) -> Option<(f64, u64)> {
	let mut distance = 0.0;
	let mut iteration = 0;
	while distance < DISTANCE_MAX && iteration < ITERATION_MAX {
		match sdf.0(ray.pos + distance * ray.dir) {
			d if d < DISTANCE_MIN => return Some((distance, iteration)),
			d => distance += d,
		}
		iteration += 1;
	}
	None
}

#[derive(Debug)]
struct State {
	scene: SDF,
}

impl State {
	fn new() -> State {
		State {
			scene: SDF::union(
				SDF::sphere().translate(Vector(0.5, 0.0, 2.0)),
				SDF::sphere().scale(0.75).translate(Vector(-1.0, 0.0, 3.0)),
			),
		}
	}
}

fn update(_state: &mut State) {}

use std::f64::consts::PI;
const FOV: f64 = PI / 2.0;
fn render(state: &State, pixels: &mut [Color; PIXELS]) {
	for x in 0..WIDTH {
		for y in 0..HEIGHT {
			let i = (x + y * HEIGHT) as usize;
			let x = x as f64 / WIDTH as f64 - 0.5;
			let y = y as f64 / HEIGHT as f64 - 0.5;
			let z = (PI * 0.5 - FOV * 0.5).tan() * 0.5;
			let ray = Ray::new(Vector::ZERO, Vector(x, y, z).normalized());
			pixels[i] = match raymarch(&state.scene, ray) {
				None => Color::BLACK,
				Some((_dist, _iter)) => Color::WHITE,
			}
		}
	}
}
