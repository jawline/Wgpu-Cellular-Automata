use glam::{Mat4, Vec3};
use winit::event::{ElementState, VirtualKeyCode};

pub struct SimpleCamera {
    pub x_off: f32,
    pub y_off: f32,

    /* A scalar that is 1. if the key is down or 0. if the key is not */
    pub a_down: f32,
    pub d_down: f32,
    pub w_down: f32,
    pub s_down: f32,
}

impl SimpleCamera {
    pub fn new() -> Self {
        Self {
            x_off: 0.,
            y_off: 0.,
            a_down: 0.,
            d_down: 0.,
            w_down: 0.,
            s_down: 0.,
        }
    }

    pub fn key(&mut self, code: VirtualKeyCode, state: ElementState) {
        use VirtualKeyCode::*;
        let down = match state {
            ElementState::Pressed => 1.,
            ElementState::Released => 0.,
        };
        match code {
            W => {
                self.w_down = down;
            }
            S => {
                self.s_down = down;
            }
            A => {
                self.a_down = down;
            }
            D => {
                self.d_down = down;
            }
            _ => {}
        }
    }

    pub fn update(&mut self, elapsed: f32) {
        let distance = 15. * elapsed;
        let x_off_delta = (self.a_down * distance) + (self.d_down * -1. * distance);
        let y_off_delta = (self.w_down * distance * -1.) + (self.s_down * distance);
        self.x_off += x_off_delta;
        self.y_off += y_off_delta;
    }

    pub fn view(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(self.x_off, self.y_off, -250.))
    }
}
