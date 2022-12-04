use glam::{Mat4, Vec3};
use winit::event::VirtualKeyCode;

pub struct Camera {
    pub pos: Vec3,
    pub rot: Vec3,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            pos: Vec3::new(0., 0., 0.),
            rot: Vec3::new(0., 0., 0.),
        }
    }

    pub fn keypress(&mut self, code: VirtualKeyCode) {
        use VirtualKeyCode::*;
        match code {
            W => {}
            S => {}
            A => {}
            D => {}
            _ => {}
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.pos)
            * Mat4::from_rotation_x(self.rot.x)
            * Mat4::from_rotation_y(self.rot.y)
            * Mat4::from_rotation_z(self.rot.z)
    }
}
