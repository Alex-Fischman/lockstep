struct VertexOutput {
    [[builtin(position)]] clip: vec4<f32>;
    [[location(0)]] frag: vec2<f32>;
};

[[stage(vertex)]]
fn vert_main(
    [[builtin(vertex_index)]] in_vertex_index: u32,
) -> VertexOutput {
    var x: f32 = -1.0;
    var y: f32 = -1.0;
    if (in_vertex_index < 2u) { x = 1.0; }
    if (in_vertex_index % 2u == 1u) { y = 1.0; }
    var out: VertexOutput;
    out.clip = vec4<f32>(x, y, 0.0, 1.0);
    out.frag = out.clip.xy;
    return out;
}

let SPHERE: i32 = 0;
fn sdf(shape_type: i32, p: vec3<f32>) -> f32 {
    if (shape_type == SPHERE) {
        return length(p) - 1.0;
    } else {
        return 0.0;
    }
}

struct Hit {
    pos: vec3<f32>;
    iter: u32;
    dist: f32;
    hit: bool;
};

let MAX_ITER: u32 = 10u;
let MAX_DIST: f32 = 1000.0;
let MIN_DIST: f32 = 0.001;
fn raymarch(eye: vec3<f32>, ray: vec3<f32>) -> Hit {
    var hit: Hit = Hit(eye, 0u, 0.0, false);
    loop {
        let dist: f32 = sdf(0, hit.pos);
        hit.iter = hit.iter + 1u;
        hit.dist = hit.dist + dist;
        if (abs(dist) < MIN_DIST) { hit.hit = true; }
        if (hit.iter > MAX_ITER || abs(dist) > MAX_DIST || abs(dist) < MIN_DIST) { break; }
        hit.pos = hit.pos + ray * dist;
    }
    return hit;
}

struct Uniforms {
    aspect_ratio: f32;
};
[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[stage(fragment)]]
fn frag_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let ray: vec3<f32> = normalize(vec3<f32>(in.frag.x, in.frag.y * uniforms.aspect_ratio, 1.0));
    let eye: vec3<f32> = vec3<f32>(0.0, 0.0, -2.0);
    let hit: Hit = raymarch(eye, ray);
    if (hit.hit) {
        return vec4<f32>(abs(hit.pos), 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}
