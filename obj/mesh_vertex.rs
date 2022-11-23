use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
pub struct Vertex {
    pub _pos: [f32; 4],
    pub _tex_coord: [f32; 3],
    pub _normal: [f32; 3],
}

impl Vertex {
    pub fn new(position: &Vec4, texture_coordinate: &Option<Vec3>, normal: &Option<Vec3>) -> Self {
        Self {
            _pos: [position.x, position.y, position.z, position.w],
            _tex_coord: match texture_coordinate {
                Some(texture_coordinate) => [
                    texture_coordinate.x,
                    texture_coordinate.y,
                    texture_coordinate.z,
                ],
                None => [0., 0., 0.],
            },
            _normal: match normal {
                Some(normal) => [normal.x, normal.y, normal.z],
                None => [0., 0., 0.],
            },
        }
    }
}
