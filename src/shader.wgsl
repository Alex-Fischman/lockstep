@vertex
fn vertex(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
	switch index {
		case 0u { return vec4<f32>(-1.0, 1.0, 0.0, 1.0); }
		case 1u { return vec4<f32>(3.0, 1.0, 0.0, 1.0); }
		case 2u { return vec4<f32>(-1.0, -3.0, 0.0, 1.0); }
		default { return vec4<f32>(0.0, 0.0, 0.0, 0.0); }
	}
}

struct Uniforms {
	window_width: f32,
	window_height: f32,
	seconds: f32,

	min_dist: f32,
	max_dist: f32,
	max_iter: u32,

	camera: Camera,
}

struct Camera {
	pos: vec3<f32>,
	dir: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct Distance {
	tag: u32, // Sphere, Plane, Union, Intersection, Exclusion, Subtraction
	x: u32,
	y: u32,
	v: vec4<f32>,
}

@group(0) @binding(1)
var<storage> distances: array<Distance>;

struct Material {
	tag: u32, // Flat
	r: f32,
	g: f32,
	b: f32,
}

@group(0) @binding(2)
var<storage> materials: array<Material>;

struct DistanceResult {
	distance: f32,
	material: u32,
}

fn distance(p: vec3<f32>) -> DistanceResult {
	var ds: array<f32, 100>; // assert: 100 <= arrayLength(distances)
	var ms: array<u32, 100>; // assert: 100 <= arrayLength(distances)
	for (var i: u32 = 0u; i < arrayLength(&distances); i++) {
		switch distances[i].tag {
			case 0u {
				let center: vec3<f32> = distances[i].v.xyz;
				let radius: f32 = distances[i].v.w;
				ds[i] = length(p - center) - radius;
				ms[i] = distances[i].x;
			}
			case 1u {
				let normal: vec3<f32> = distances[i].v.xyz;
				let offset: f32 = distances[i].v.w;
				ds[i] = dot(p, normal) - offset;
				ms[i] = distances[i].x;
			}
			default {
				let x = ds[distances[i].x];
				let y = ds[distances[i].y];
				switch distances[i].tag {
					case 2u {
						if x < y { ds[i] = x; ms[i] = ms[distances[i].x]; }
						else     { ds[i] = y; ms[i] = ms[distances[i].y]; }
					}
					case 3u {
						if x > y { ds[i] = x; ms[i] = ms[distances[i].x]; }
						else     { ds[i] = y; ms[i] = ms[distances[i].y]; }
					}
					case 4u {
						ds[i] = max(min(x, y), -max(x, y));
						// todo: material
					}
					case 5u {
						if -x > y { ds[i] = -x; ms[i] = ms[distances[i].x]; }
						else      { ds[i] =  y; ms[i] = ms[distances[i].y]; }
					}
					default {}
				}
			}
		}
	}
	var out: DistanceResult;
	out.distance = ds[arrayLength(&distances) - 1u];
	out.material = ms[arrayLength(&distances) - 1u];
	return out;
}

struct RaymarchResult {
	tag: u32, // Hit, WentTooFar, TookTooLong
	steps: u32,
	material: u32,
	point: vec3<f32>,
}

fn raymarch(pos: vec3<f32>, dir: vec3<f32>) -> RaymarchResult {
	var out: RaymarchResult;
	out.steps = 0u;
	var accum = 0.0;

	while out.steps < uniforms.max_iter {
		out.point = pos + dir * accum;
		let distanceResult = distance(out.point);

		if distanceResult.distance < uniforms.min_dist {
			out.tag = 0u;
			out.material = distanceResult.material;
			return out;
		} else if distanceResult.distance > uniforms.max_dist {
			out.tag = 1u;
			return out;
		}
		out.steps += 1u;
		accum += distanceResult.distance;
	}

	out.tag = 2u;
	return out;
}

@fragment
fn fragment(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
	var xy = pos.xy / vec2<f32>(uniforms.window_width, uniforms.window_height) * 2.0 - 1.0;
	xy.y *= -1.0;
	if uniforms.window_height < uniforms.window_width {
		xy.x *= uniforms.window_width / uniforms.window_height;
	} else {
		xy.y *= uniforms.window_height / uniforms.window_width;
	}
	let xyz = vec3<f32>(xy.x, xy.y, 0.0);

	let raymarch = raymarch(uniforms.camera.pos + xyz, vec3<f32>(0.0, 0.0, 1.0));
	switch raymarch.tag {
		case 0u {
			switch materials[raymarch.material].tag {
				case 0u {
					let m = materials[raymarch.material];
					return vec4<f32>(m.r, m.g, m.b, 1.0);
				}
				default { return vec4<f32>(1.0, 0.0, 1.0, 1.0); }
			}
		}
		case 1u { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
		case 2u { return vec4<f32>(0.0, 0.0, 1.0, 1.0); }
		default { return vec4<f32>(1.0, 0.0, 1.0, 1.0); }
	}
}
