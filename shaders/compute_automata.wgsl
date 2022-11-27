@group(0)
@binding(0)
var<storage, read> automata_dim: vec3<u32>;

@group(0)
@binding(1)
var<storage, read_write> input_tensor: array<u32>;

@group(0)
@binding(2)
var<storage, read_write> output_tensor: array<u32>;

fn neighbors(id: u32) -> u32 {
  // TODO: This can go out of bounds on 0 or dim indices, we should discard
  // the outermost rows/columns/layers instead.
  // TODO: Precompute and pass these in as the global id arguments?
  // Sign everything so we can have negative strides
  let id: i32 = i32(id);
  let Z_STRIDE: i32 = i32(automata_dim.x * automata_dim.y);
  let Y_STRIDE: i32 = i32(automata_dim.x);

  var result: u32 = u32(0);

  for (var z: i32 = 0; z < 3; z += 1) {
    let id = id + (Z_STRIDE * (z - 1));
    for (var y: i32 = 0; y < 3; y += 1) { 
      let id: i32 = id + (Y_STRIDE * (y - 1));
      for (var x: i32 = 0; x < 3; x += 1) {
        result += u32(input_tensor[id]);
      }
    }
  }

  return result;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

  let num_neighbors: u32 = neighbors(global_id.x) - input_tensor[global_id.x];

  var result: u32 = u32(0);

  if /* (num_neighbors >= 6u && num_neighbors <= 8u)
  || */ (num_neighbors >= 3u && num_neighbors <= 4u) {
    result = u32(1);
  } else {
    result = u32(0);
  };

  output_tensor[global_id.x] = result;
}
