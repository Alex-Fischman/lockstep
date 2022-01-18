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
    var pos = v;
    var dists: array<f32, 2>;
    var dists_i = 0;
    for (var i = 0; i < textureDimensions(texture); i = i + 1) {
        switch (u32(textureLoad(texture, i, 0).r)) {
            case 0: {
                i = i + 1;
                let r = textureLoad(texture, i, 0).r;
                dists[dists_i] = length(pos) - 1.0;
                dists_i = dists_i + 1;
                pos = v;
            }
            case 1: {
                let x = textureLoad(texture, i + 1, 0).r;
                let y = textureLoad(texture, i + 2, 0).r;
                let z = textureLoad(texture, i + 3, 0).r;
                i = i + 3;
                pos = pos - vec3<f32>(x, y, z);
            }
            case 2: {
                dists[0] = min(dists[0], dists[1]);
                dists_i = 0;
            }
            case 3: {
                dists[0] = max(dists[0], dists[1]);
                dists_i = 0;
            }
            case 4: {
                dists[0] = max(-dists[0], dists[1]);
                dists_i = 0;
            }
            default: {}
        } 
    }
    return dists[0];
}

[[stage(fragment)]]
fn frag_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let pos = vec3<f32>(in.frag, 0.0);
    let dist = sdf(pos);
    return vec4<f32>(dist, 0.0, 0.0, 1.0);
}
