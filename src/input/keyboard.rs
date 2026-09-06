use std::collections::HashSet;
use winit::keyboard::KeyCode;

#[derive(Default, Clone, PartialEq)]
pub struct KeyboardState {
    pressed_keys: HashSet<KeyCode>,
}

impl KeyboardState {
    pub fn set_key_pressed(&mut self, keycode: KeyCode, is_pressed: bool) {
        match is_pressed {
            true => self.pressed_keys.insert(keycode),
            false => self.pressed_keys.remove(&keycode),
        };
    }

    #[allow(unused)]
    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        self.pressed_keys.contains(&key_code)
    }
}
