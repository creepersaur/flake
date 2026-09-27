use std::collections::HashSet;
use winit::keyboard::KeyCode;

#[derive(Default, Clone, PartialEq)]
pub struct KeyboardState {
    pressed_keys: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
}

impl KeyboardState {
    pub fn set_key_pressed(&mut self, keycode: KeyCode, is_pressed: bool) {
        match is_pressed {
            true => {
                self.pressed_keys.insert(keycode);
                self.just_pressed.insert(keycode)
            }
            false => {
                self.pressed_keys.remove(&keycode);
                self.just_released.insert(keycode)
            }
        };
    }

    #[allow(unused)]
    pub fn is_key_down(&self, key_code: KeyCode) -> bool {
        self.pressed_keys.contains(&key_code)
    }

    #[allow(unused)]
    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        self.just_pressed.contains(&key_code)
    }

    #[allow(unused)]
    pub fn is_key_released(&self, key_code: KeyCode) -> bool {
        self.just_released.contains(&key_code)
    }

    pub fn clear_just_pressed(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}
