let LIGHT_DIRECTION: vec3<f32> = vec3<f32>(0., 0., 0.);

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

struct VertexOutput {
    @builtin(position) proj_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec4<f32>
};

@vertex
fn vs_main(@location(0) position: vec4<f32>, @location(2) normal: vec3<f32>) -> VertexOutput {
    var result: VertexOutput;
    result.world_normal = normal;
    result.world_position = position;
    result.proj_position = position * transform;
    return result;
}

@fragment
fn fs_main(@location(0) normal: vec3<f32>, @location(1) world_position: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
