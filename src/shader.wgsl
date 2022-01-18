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

[[group(0), binding(0)]]
var texture: texture_1d<f32>;

fn sdf(v: vec3<f32>) -> f32 {
    var translation = vec3<f32>(0.0);
    var dist = 0.0;
    var temp = 0.0;
    var dist_used = false;
    for (var i = 0; i < textureDimensions(texture); i = i + 1) {
        let op = textureLoad(texture, i, 0).r;
        if (op == 0.0) {
            i = i + 1;
            let r = textureLoad(texture, i, 0).r;
            let d = length(v - translation) - r;
            if (dist_used) { temp = d; } else { dist = d; dist_used = true; }
            translation = vec3<f32>(0.0);
        } else if (op == 1.0) {
            let x = textureLoad(texture, i + 1, 0).r;
            let y = textureLoad(texture, i + 2, 0).r;
            let z = textureLoad(texture, i + 3, 0).r;
            i = i + 3;
            translation = translation + vec3<f32>(x, y, z);
        } else if (op == 2.0) {
            dist = min(dist, temp);
        } else if (op == 3.0) {
            dist = max(dist, temp);
        } else if (op == 4.0) {
            dist = max(-dist, temp);
        } else {
            dist = 1.0 / 0.0;
        }
    }
    return dist;
}

struct Raymarch {
    pos: vec3<f32>;
    dir: vec3<f32>;
    iter: u32;
    dist: f32;
};

let MIN_DIST = 0.01;
let MAX_DIST = 1000.0;
let MAX_ITER = 10u;
fn raymarch(pos: vec3<f32>, dir: vec3<f32>) -> Raymarch {
    var out = Raymarch(pos, normalize(dir), 0u, 0.0);
    loop {
        let dist = sdf(out.pos);
        if (abs(dist) < MIN_DIST || abs(dist) > MAX_DIST || out.iter > MAX_ITER) { break; }
        out.iter = out.iter + 1u;
        out.dist = out.dist + dist;
        out.pos  = out.pos  + out.dir * dist;
    }
    return out;
}

[[stage(fragment)]]
fn frag_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let dir = vec3<f32>(0.0, 0.0, 1.0);
    let pos = vec3<f32>(in.frag, -5.0);
    let ray = raymarch(pos, dir);
    return vec4<f32>(ray.pos, 1.0);
    // let uv = in.frag * 0.5 + 0.5;
    // let t = textureLoad(texture, i32(uv.x * f32(textureDimensions(texture))), 0);
    // return vec4<f32>(uv, t.r, 1.0);
}
