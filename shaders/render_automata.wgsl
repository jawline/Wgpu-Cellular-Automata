@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@group(1)
@binding(0)
var<storage, read> automata_dim: vec3<u32>;

@group(1)
@binding(1)
var<storage, read_write> input_tensor: array<u32>;

@group(1)
@binding(2)
var<storage, read_write> output_tensor: array<u32>;

struct VertexOutput {
    @builtin(position) proj_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) texture_coordinate: vec3<f32>
};

fn index_to_position(index: u32) -> vec4<f32> {
    let x = f32(i32(index) - 1);
    let y = f32(i32(index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var result: VertexOutput;
    result.world_normal = vec3<f32>(0., 0., 0.);
    result.world_position = index_to_position(vertex_index % u32(3));
    result.texture_coordinate = vec3<f32>(1., 1., 1.);
    result.proj_position = transform * result.world_position;
    return result;
}

@fragment
fn fs_main(
  @location(0) normal: vec3<f32>,
  @location(1) world_position: vec4<f32>,
  @location(2) texture_coordinate: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 0.5);
}
