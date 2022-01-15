[[stage(vertex)]]
fn vert_main([[builtin(vertex_index)]] i: u32) -> [[builtin(position)]] vec4<f32> {
    switch (i) {
        case 0:  { return vec4<f32>(-1.0,  1.0, 0.0, 1.0); }
        case 1:  { return vec4<f32>(-1.0, -1.0, 0.0, 1.0); }
        case 2:  { return vec4<f32>( 1.0,  1.0, 0.0, 1.0); }
        case 3:  { return vec4<f32>(-1.0, -1.0, 0.0, 1.0); }
        case 4:  { return vec4<f32>( 1.0,  1.0, 0.0, 1.0); }
        case 5:  { return vec4<f32>( 1.0, -1.0, 0.0, 1.0); }
        default: { return vec4<f32>( 0.0,  0.0, 0.0, 0.0); }
    }
}

[[stage(fragment)]]
fn frag_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
