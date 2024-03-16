/// WAT3RS Project
/// `File` render/texture.rs
/// `Description` Input impementation module
/// `Author` TioT2
/// `Last changed` 17.02.2024

use std::collections::BTreeMap;

#[derive(Copy, Clone, PartialEq, Eq)]
struct KeyState {
    pressed: bool,
    changed: bool,
} // struct KState

pub type KeyCode = winit::keyboard::KeyCode;

pub struct State {
    keys: BTreeMap<KeyCode, KeyState>,
}

impl State {
    fn get_key_state(&self, key: KeyCode) -> KeyState {
        if let Some(state) = self.keys.get(&key) {
            *state
        } else {
            KeyState { pressed: false, changed: false }
        }
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.get_key_state(key).pressed
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        !self.get_key_state(key).pressed
    }

    pub fn is_key_clicked(&self, key: KeyCode) -> bool {
        let state = self.get_key_state(key);
        state.pressed && state.changed
    }
}

pub struct Input {
    state: State,
}

impl Input {
    pub fn new() -> Self {
        Self {
            state: State {
                keys: BTreeMap::new(),
            },
        }
    }

    pub fn on_key_state_change(&mut self, key: KeyCode, is_pressed: bool) {
        if let Some(key_state) = self.state.keys.get_mut(&key) {
            key_state.pressed = is_pressed;
            key_state.changed = true;
        } else {
            self.state.keys.insert(key, KeyState { pressed: is_pressed, changed: true });
        }
    }

    pub fn clear_changed(&mut self) {
        for (_, state) in &mut self.state.keys {
            state.changed = false;
        }
    }

    pub fn get_state<'a>(&'a self) -> &'a State {
        &self.state
    }
}