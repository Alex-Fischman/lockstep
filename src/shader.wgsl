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
var texture: texture_2d<f32>;

[[stage(fragment)]]
fn frag_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let uv = in.frag * 0.5 + 0.5;
    return textureLoad(texture, vec2<i32>(uv * vec2<f32>(textureDimensions(texture))), 0);
}
