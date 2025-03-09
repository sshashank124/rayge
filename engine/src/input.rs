use std::collections::HashSet;

use winit::{
    event::{ElementState, MouseButton},
    keyboard::KeyCode,
};

#[derive(Default)]
pub struct State {
    keys: HashSet<KeyCode>,
    buttons: HashSet<MouseButton>,
}

impl State {
    pub fn handle_key(&mut self, key_code: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => self.keys.insert(key_code),
            ElementState::Released => self.keys.remove(&key_code),
        };
    }

    pub fn handle_mouse_button(&mut self, mouse_button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => self.buttons.insert(mouse_button),
            ElementState::Released => self.buttons.remove(&mouse_button),
        };
    }
}
