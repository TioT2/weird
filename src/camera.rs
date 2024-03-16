use crate::math::*;

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub location: Vec2f,
    pub height: f32,
    pub rotation: f32,
    pub direction: Vec2f,
    pub right: Vec2f,

    location_dot_direction: f32,
    location_dot_right: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            location: Vec2f { x: 0.0, y: 0.0 },
            height: 0.5,
            rotation: 0.0,
            direction: Vec2f { x: 1.0, y: 0.0 },
            right: Vec2f { x: 0.0, y: -1.0 },
            location_dot_direction: 0.0,
            location_dot_right: 0.0,
        }
    }

    pub fn set_location(&mut self, location: Vec2f, height: f32, rotation: f32) {
        self.location = location;
        self.rotation = rotation;
        self.height = height;

        self.direction = Vec2f {
            x: self.rotation.cos(),
            y: self.rotation.sin(),
        };

        self.right = Vec2f {
            x: self.direction.y,
            y: -self.direction.x,
        };

        self.location_dot_direction = self.location.x * self.direction.x + self.location.y * self.direction.y;
        self.location_dot_right     = self.location.x * self.right.x     + self.location.y * self.right.y    ;
    }

    pub fn to_space(&self, p: Vec2f) -> Vec2f {
        Vec2f {
            x: p.x * self.right.x     + p.y * self.right.y     - self.location_dot_right,
            y: p.x * self.direction.x + p.y * self.direction.y - self.location_dot_direction,
        }
    }
}
