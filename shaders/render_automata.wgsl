struct VertexOutput {
    @builtin(position) proj_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) texture_coordinate: vec3<f32>
};

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@group(1)
@binding(0)
var<storage, read> automata_dim: vec3<u32>;

@group(1)
@binding(1)
var<storage, read> input_tensor: array<u32>;

let NUM_VERTICES: u32 = 3u;

fn index_to_position(index: u32) -> vec4<f32> {
    let x = f32(i32(index) - 1);
    let y = f32(i32(index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

fn automata_id_to_offset(id: u32, automata_state: u32) -> vec4<f32> {
    let automatas_in_layer: u32 = automata_dim.x * automata_dim.y;
    let z = id / automatas_in_layer;

    let id: u32 = id % automatas_in_layer;
    let y = id / automata_dim.y;

    let id: u32  = id % automata_dim.y;    
    let x = id;

    if automata_state == 0u {
      // TODO: I think there must be a better way of discarding
      return vec4<f32>(-9999999., -9999999., -999999., 0.);
    } else {
      return vec4<f32>(f32(x), f32(y), f32(z), 1.0);
    }
}
    

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {

    // The draw call will run a vertex shader on each block
    // of the automata NUM_VERTICES times so we choose to draw
    // or not draw a valid shape at each point.
    let automata_id = vertex_index / NUM_VERTICES;
    let vertex_id = vertex_index % NUM_VERTICES;
    let automata_state: u32 = input_tensor[automata_id];

    let position_offset = automata_id_to_offset(automata_id, automata_state);
    let position: vec4<f32> = index_to_position(vertex_id);
  
    var result: VertexOutput;
    result.world_position = position;
    result.world_normal = vec3<f32>(0., 0., 0.);
    result.texture_coordinate = vec3<f32>(1., 1., 1.);
    result.proj_position = transform * (position_offset + position);
    return result;
}

@fragment
fn fs_main(
  @location(0) normal: vec3<f32>,
  @location(1) world_position: vec4<f32>,
  @location(2) texture_coordinate: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
