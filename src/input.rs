/// WEIRD Project
/// `File` input.rs
/// `Description` Input impementation module
/// `Author` TioT2
/// `Last changed` 17.03.2024

use std::collections::BTreeMap;
use crate::math::Vec2f;

#[derive(Copy, Clone, PartialEq, Eq)]
struct KeyState {
    pub pressed: bool,
    pub changed: bool,
} // struct KState

/// Keycode representation structure
pub type KeyCode = winit::keyboard::KeyCode;

/// Keystate getting function
pub struct State {
    keys: BTreeMap<KeyCode, KeyState>,
    mouse_location: Vec2f,
    mouse_motion: Vec2f,
} // struct State

impl State {
    fn get_key_state(&self, key: KeyCode) -> KeyState {
        if let Some(state) = self.keys.get(&key) {
            *state
        } else {
            KeyState { pressed: false, changed: false }
        }
    } // fn get_key_state

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.get_key_state(key).pressed
    } // fn is_key_pressed

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        !self.get_key_state(key).pressed
    } // fn is_key_released

    pub fn is_key_clicked(&self, key: KeyCode) -> bool {
        let state = self.get_key_state(key);
        state.pressed && state.changed
    } // fn is_key_clicked

    pub fn get_mouse_location(&self) -> Vec2f {
        self.mouse_location
    } // fn get_mouse_location

    pub fn get_mouse_motion(&self) -> Vec2f {
        self.mouse_motion
    } // fn get_mouse_motion

    // pub fn get_mouse_wheel(&self) -> isize {
    //     self.mouse_wheel
    // } // fn get_mouse_wheel
    //
    // pub fn get_mosue_wheel_motion(&self) -> isize {
    //     self.mouse_wheel_motion
    // } // fn get_mouse_wheel_motion
} // impl State

// struct Input
pub struct Input {
    state: State,
} // struct Input

impl Input {
    pub fn new() -> Self {
        Self {
            state: State {
                keys: BTreeMap::new(),
                mouse_location: Vec2f { x: 0.0, y: 0.0 },
                mouse_motion: Vec2f { x: 0.0, y: 0.0 },
            },
        }
    } // fn new

    pub fn on_key_state_change(&mut self, key: KeyCode, is_pressed: bool) {
        if let Some(key_state) = self.state.keys.get_mut(&key) {
            key_state.pressed = is_pressed;
            key_state.changed = true;
        } else {
            self.state.keys.insert(key, KeyState { pressed: is_pressed, changed: true });
        }
    } // fn on_key_state_change

    pub fn on_mouse_move(&mut self, new_position: Vec2f) {
        self.state.mouse_motion.x += new_position.x - self.state.mouse_location.x;
        self.state.mouse_motion.y += new_position.y - self.state.mouse_location.y;
        self.state.mouse_location = new_position;
    }

    // pub fn on_mouse_wheel(&mut self, wheel: isize) {
    //     self.state.mouse_wheel_motion += wheel - self.state.mouse_wheel;
    //     self.state.mouse_wheel = wheel;
    // }

    // Changed parameters clearing function
    pub fn clear_changed(&mut self) {
        for (_, state) in &mut self.state.keys {
            state.changed = false;
        }
        self.state.mouse_motion = Vec2f { x: 0.0, y: 0.0 };
    } // fn clear_changed

    pub fn get_state<'a>(&'a self) -> &'a State {
        &self.state
    } // fn get_state
} // impl Input

// fiel input.rs