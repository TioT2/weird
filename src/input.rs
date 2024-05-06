/// WEIRD Project
/// `File` input.rs
/// `Description` Input impementation module
/// `Author` TioT2
/// `Last changed` 05.05.2024

use std::collections::BTreeMap;
use crate::math::Vec2f;

/// Single key state
#[derive(Copy, Clone, PartialEq, Eq)]
struct KeyState {
    /// Is key pressed
    pub pressed: bool,
    /// Is key state changed during previous frame
    pub changed: bool,
} // struct KState

/// Keycode representation structure
pub type KeyCode = winit::keyboard::KeyCode;

/// Input state representation structure
pub struct State {
    keys: BTreeMap<KeyCode, KeyState>,
    mouse_location: Vec2f,
    mouse_motion: Vec2f,
} // struct State

impl State {
    /// Key state getting function
    /// * `key` - keycode to get state of
    /// * Returns key state
    fn get_key_state(&self, key: KeyCode) -> KeyState {
        if let Some(state) = self.keys.get(&key) {
            *state
        } else {
            KeyState { pressed: false, changed: false }
        }
    } // fn get_key_state

    /// Is key pressed checking function
    /// * `key` - key to check state of
    /// * Returns true if key is pressed
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.get_key_state(key).pressed
    } // fn is_key_pressed

    /// Is key released checking function
    /// * `key` - key to check state of
    /// * Returns true if key's released
    pub fn is_key_released(&self, key: KeyCode) -> bool {
        !self.get_key_state(key).pressed
    } // fn is_key_released

    /// Is key clicked checking function
    /// * `key` - key to check state of
    /// * Returns true if key's clicked
    pub fn is_key_clicked(&self, key: KeyCode) -> bool {
        let state = self.get_key_state(key);
        state.pressed && state.changed
    } // fn is_key_clicked

    /// Mouse location getting function
    /// * Returns mouse location as Vec2f
    pub fn get_mouse_location(&self) -> Vec2f {
        self.mouse_location
    } // fn get_mouse_location

    /// Mouse delta getting function
    /// * Returns mouse motion
    pub fn get_mouse_motion(&self) -> Vec2f {
        self.mouse_motion
    } // fn get_mouse_motion
} // impl State

// Input getting function
pub struct Input {
    state: State,
} // struct Input

impl Input {
    /// New input construction function
    /// * Returns newly-created input
    pub fn new() -> Self {
        Self {
            state: State {
                keys: BTreeMap::new(),
                mouse_location: Vec2f { x: 0.0, y: 0.0 },
                mouse_motion: Vec2f { x: 0.0, y: 0.0 },
            },
        }
    } // fn new

    /// Key state change callback
    /// * `key` - keycode
    /// * `is_pressed` - changed key state
    pub fn on_key_state_change(&mut self, key: KeyCode, is_pressed: bool) {
        if let Some(key_state) = self.state.keys.get_mut(&key) {
            key_state.pressed = is_pressed;
            key_state.changed = true;
        } else {
            self.state.keys.insert(key, KeyState { pressed: is_pressed, changed: true });
        }
    } // fn on_key_state_change

    /// Mouse motion callback
    /// * `new_position` - new mouse position
    pub fn on_mouse_move(&mut self, new_position: Vec2f) {
        self.state.mouse_motion.x += new_position.x - self.state.mouse_location.x;
        self.state.mouse_motion.y += new_position.y - self.state.mouse_location.y;
        self.state.mouse_location = new_position;
    } // fn on_mouse_move

    // Changed parameters clearing function
    pub fn clear_changed(&mut self) {
        for (_, state) in &mut self.state.keys {
            state.changed = false;
        }
        self.state.mouse_motion = Vec2f { x: 0.0, y: 0.0 };
    } // fn clear_changed

    /// State getting function
    /// * Returns input state reference
    pub fn get_state<'a>(&'a self) -> &'a State {
        &self.state
    } // fn get_state
} // impl Input

// file input.rs
