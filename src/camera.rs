/// WEIRD Project
/// `File` camera.rs
/// `Description` Camera utility impementation module
/// `Author` TioT2
/// `Last changed` 05.05.2024

use crate::math::*;

/// Camera utility representation structure
#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub location: Vec2f,
    pub height: f32,
    pub rotation: f32,
    pub direction: Vec2f,
    pub right: Vec2f,

    location_dot_direction: f32,
    location_dot_right: f32,
} // struct Camera

/// Camera state represetnation structure
pub struct State {
    /// Camera location
    pub location: Vec2f,
    /// Camera height
    pub height: f32,
    /// Camera rotation
    pub rotation: f32,
} // pub struct State

impl Camera {
    /// New camera create function
    /// * Returns newly created camera
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
    } // fn new

    /// Camera location setting function
    /// * `location` - camera location
    /// * `height` - camera location height
    /// * `rotation` - camera rotation angle (ccw)
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
    } // fn set_location

    /// Point from global to camera space transformation function
    /// * `p` - point to transform
    /// * Returns transformed point
    pub fn to_space(&self, p: Vec2f) -> Vec2f {
        Vec2f {
            x: p.x * self.right.x     + p.y * self.right.y     - self.location_dot_right,
            y: p.x * self.direction.x + p.y * self.direction.y - self.location_dot_direction,
        }
    } // fn to_space
} // impl Camera

// file camera.rs
