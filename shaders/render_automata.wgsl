struct VertexOutput {
    @builtin(position) proj_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) texture_coordinate: vec4<f32>
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

let NUM_VERTICES: u32 = 36u;

fn index_to_position(index: u32) -> vec4<f32> {
    let triangle_id: u32 = index / 3u; 
    let index = index % 3u;

    var x: f32 = 0.;
    var y: f32 = 0.;
    var z: f32 = 0.;

    if triangle_id == 0u {
      // 0, 0
      // 0, 1
      // 1, 0
      x = f32(i32(index / 2u));
      y = f32(i32(index & 1u)); 
      z = 0.;
    } else if triangle_id == 1u {
      // 1, 0
      // 1, 1
      // 0, 0
      x = f32(1 - i32(index / 2u));
      y = f32(i32(index >= 1u)); 
      z = 0.;
    } else if triangle_id == 2u {
      x = f32(i32(index / 2u));
      y = f32(i32(index & 1u)); 
      z = 1.;
    } else if triangle_id == 3u {
      x = f32(1 - i32(index / 2u));
      y = f32(i32(index >= 1u));
      z = 1.;
    } else if triangle_id == 4u {
      x = f32(i32(index / 2u));
      z = f32(i32(index & 1u)); 
      y = 0.;
    } else if triangle_id == 5u {
      x = f32(1 - i32(index / 2u));
      z = f32(i32(index >= 1u)); 
      y = 0.;
    } else if triangle_id == 6u {
      x = f32(i32(index / 2u));
      z = f32(i32(index & 1u)); 
      y = 1.;
    } else if triangle_id == 7u { 
      x = f32(1 - i32(index / 2u));
      z = f32(i32(index >= 1u)); 
      y = 1.;
    } else if triangle_id == 8u {
      y = f32(i32(index / 2u));
      z = f32(i32(index & 1u)); 
      x = 0.;
    } else if triangle_id == 9u { 
      y = f32(1 - i32(index / 2u));
      z = f32(i32(index >= 1u)); 
      x = 0.;
    } else if triangle_id == 10u {
      y = f32(i32(index / 2u));
      z = f32(i32(index & 1u)); 
      x = 1.;
    } else if triangle_id == 11u { 
      y = f32(1 - i32(index / 2u));
      z = f32(i32(index >= 1u)); 
      x = 1.;
    }

    return vec4<f32>(x, y, z, 1.0);
}

fn automata_id_to_offset(id: u32, automata_state: u32) -> vec4<f32> {
    let automatas_in_layer: u32 = automata_dim.x * automata_dim.y;
    let z = id / automatas_in_layer;

    let id: u32 = id % automatas_in_layer;
    let y = id / automata_dim.x;

    let id: u32  = id % automata_dim.x;    
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
    let raw_position: vec4<f32> = index_to_position(vertex_id);
    let position: vec4<f32> = raw_position - vec4<f32>(
      f32(automata_dim.x) / 2.,
      f32(automata_dim.y) / 2.,
      f32(automata_dim.z) / 2.,
      1.
    );
  
    var result: VertexOutput;
    result.world_position = position;
    result.world_normal = vec3<f32>(0., 0., 0.);
    result.texture_coordinate = raw_position;
    result.proj_position = transform * (position_offset + position);
    return result;
}

@fragment
fn fs_main(
  @location(0) normal: vec3<f32>,
  @location(1) world_position: vec4<f32>,
  @location(2) texture_coordinate: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(texture_coordinate.x * 0.9, texture_coordinate.y * 0.9, texture_coordinate.z * 0.9, 1.0);
}
