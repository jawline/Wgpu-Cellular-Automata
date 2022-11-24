let LIGHT_DIRECTION: vec3<f32> = vec3<f32>(0., 0., 0.);

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) proj_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) texture_coordinate: vec3<f32>
};

@vertex
fn vs_main(@location(0) position: vec4<f32>, @location(1) texture: vec3<f32>, @location(2) normal: vec3<f32>, instance: InstanceInput) -> VertexOutput {

    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var result: VertexOutput;
    result.world_normal = normal;
    result.world_position = position;
    result.texture_coordinate = texture;
    result.proj_position = transform * model_matrix * position;
    return result;
}

@fragment
fn fs_main(
  @location(0) normal: vec3<f32>,
  @location(1) world_position: vec4<f32>,
  @location(2) texture_coordinate: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 1.0) * vec4<f32>(texture_coordinate.x, 0., texture_coordinate.y, 0.);
}
