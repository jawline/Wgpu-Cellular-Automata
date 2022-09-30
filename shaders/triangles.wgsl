let ROW_SIZE: f32 = 0.1;
let ROW_VERTEX_COUNT: u32 = 30u;

let LIGHT_DIRECTION: vec3<f32> = vec3<f32>(0., 0., 0.);


@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(in_vertex_index & ROW_VERTEX_COUNT) - 1) * 0.05;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.05;
    return transform * vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
