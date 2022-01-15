struct VertexOutput {
    [[builtin(position)]] clip: vec4<f32>;
    [[location(0)]]       frag: vec2<f32>;
};

[[stage(vertex)]]
fn vert_main([[builtin(vertex_index)]] i: u32) -> VertexOutput {
    var pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    switch (i) {
        case 0:  { pos = vec4<f32>(-1.0,  1.0, 0.0, 1.0); }
        case 1:  { pos = vec4<f32>(-1.0, -1.0, 0.0, 1.0); }
        case 2:  { pos = vec4<f32>( 1.0,  1.0, 0.0, 1.0); }
        case 3:  { pos = vec4<f32>(-1.0, -1.0, 0.0, 1.0); }
        case 4:  { pos = vec4<f32>( 1.0,  1.0, 0.0, 1.0); }
        case 5:  { pos = vec4<f32>( 1.0, -1.0, 0.0, 1.0); }
        default: { pos = vec4<f32>( 0.0,  0.0, 0.0, 0.0); }
    }
    return VertexOutput(pos, pos.xy);
}

fn sphere(p: vec3<f32>) -> f32 {
    return length(p) - 1.0;
}

fn sdf(p: vec3<f32>) -> f32 {
    return min(sphere(p), sphere(p - vec3<f32>(2.0, 0.0, 0.0)));
}

struct Hit {
    pos: vec3<f32>;
    iter: u32;
    dist: f32;
};

let MIN_DIST = 0.01;
let MAX_DIST = 1000.0;
let MAX_ITER = 10u;
fn raymarch(eye: vec3<f32>, dir: vec3<f32>) -> Hit {
    var out = Hit(eye, 0u, 0.0);
    loop {
        let dist = sdf(out.pos);
        if (abs(dist) < MIN_DIST || abs(dist) > MAX_DIST || out.iter > MAX_ITER) { break; }
        out.iter = out.iter + 1u;
        out.dist = min(out.dist, dist);
        out.pos  = out.pos  + normalize(dir) * dist;
    }
    return out;
}

struct Uniforms {
    aspect_ratio: f32;
};
[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[stage(fragment)]]
fn frag_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let eye = vec3<f32>(in.frag.x, in.frag.y * uniforms.aspect_ratio, -5.0);
    let hit = raymarch(eye * 5.0, vec3<f32>(0.0, 0.0, 1.0));
    if (abs(sdf(hit.pos)) < MIN_DIST) { return vec4<f32>(1.0); }
    else { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
}
