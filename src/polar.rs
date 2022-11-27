use std::f32::{self, consts::PI};

pub struct Polar {
    radius: f32,
    angle: f32,
    cycles_per_second: f32,
}

impl Polar {
    pub fn new(radius: f32, angle: f32, cycles_per_second: f32) -> Self {
        Self {
            radius,
            angle,
            cycles_per_second,
        }
    }

    pub fn update(&mut self, elapsed_seconds: f32) {
        self.angle += self.cycles_per_second * elapsed_seconds;
        self.angle %= PI * 2.;
    }

    pub fn position(&self) -> (f32, f32) {
        (
            self.radius * f32::cos(self.angle),
            self.radius * f32::sin(self.angle),
        )
    }
}
