@group(0)
@binding(0)
var<storage, read> automata_dim: vec3<u32>;

@group(0)
@binding(1)
var<storage, read_write> input_tensor: array<u32>;

@group(0)
@binding(2)
var<storage, read_write> output_tensor: array<u32>;

fn xyz_to_id(xyz: vec3<u32>) -> u32 {
    let z = (xyz.z * (automata_dim.x * automata_dim.y));
    let y = (xyz.y * automata_dim.x);
    return xyz.x + y + z;
}

fn neighbors(offset: vec3<u32>) -> u32 {
  // Skip the boundaries to avoid out of bounds weirdness
  if offset.x == 0u || offset.y == 0u || offset.z == 0u ||
     offset.x == automata_dim.x - 1u || offset.y == automata_dim.y - 1u ||
     offset.z == automata_dim.z - 1u {
    return 0u;
  } else {
    var result: u32 = 0u;

    for (var z: u32 = 0u; z < 3u; z += 1u) {
      for (var y: u32 = 0u; y < 3u; y += 1u) { 
        for (var x: u32 = 0u; x < 3u; x += 1u) {
          let point_id: u32 = xyz_to_id(vec3<u32>(x, y, z) + offset - vec3<u32>(1u, 1u, 1u));
          result += u32(input_tensor[point_id]);
        }
      }
    }

    return result;
  }
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) pos: vec3<u32>) {

  let id: u32 = xyz_to_id(pos);

  let currently_alive = input_tensor[id] > 0u;
  let num_neighbors: u32 = neighbors(pos) - input_tensor[id];

  var result: u32 = 0u;

  if currently_alive {
    if (num_neighbors >= 7u && num_neighbors <= 14u) {
      result = 1u;
    }
  } else {
    if num_neighbors == 8u {
      result = 1u;
    }
  }
 
  output_tensor[id] = result;
}
