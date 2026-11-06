use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use winit::keyboard::KeyCode;

#[derive(Default, Clone)]
pub struct KeyboardState {
    pressed_keys: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,

    global_pressed_keys: HashSet<KeyCode>,
    global_just_pressed: HashSet<KeyCode>,
    global_just_released: HashSet<KeyCode>,

    pub last_key_name: Option<String>,
    pub typed_string: String,

    pub blocked_keys: Arc<RwLock<HashSet<KeyCode>>>,
}

impl KeyboardState {
    pub fn get_pressed_keys(&mut self) -> impl Iterator<Item = KeyCode> {
        self.pressed_keys.iter().copied()
    }

    pub fn get_pressed_keys_global(&mut self) -> impl Iterator<Item = KeyCode> {
        self.global_pressed_keys.iter().copied()
    }

    pub fn set_key_pressed(&mut self, keycode: KeyCode, is_pressed: bool) {
        if is_pressed {
            self.pressed_keys.insert(keycode);
            self.just_pressed.insert(keycode);
        } else {
            self.pressed_keys.remove(&keycode);
            self.just_released.insert(keycode);
        }
    }

    pub fn set_key_pressed_global(
        &mut self,
        keycode: KeyCode,
        name: Option<String>,
        is_pressed: bool,
    ) {
        if is_pressed {
            self.global_pressed_keys.insert(keycode);
            self.global_just_pressed.insert(keycode);

            if let Some(n) = name
                .as_deref()
                .filter(|s| !s.is_empty() && s.chars().all(|c| !c.is_control()))
            {
                self.typed_string.push_str(n);
            }
            self.last_key_name = name;
        } else {
            self.global_pressed_keys.remove(&keycode);
            self.global_just_released.insert(keycode);
        }
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

    #[allow(unused)]
    pub fn is_key_down_global(&self, key_code: KeyCode) -> bool {
        self.global_pressed_keys.contains(&key_code)
    }
    #[allow(unused)]
    pub fn is_key_pressed_global(&self, key_code: KeyCode) -> bool {
        self.global_just_pressed.contains(&key_code)
    }
    #[allow(unused)]
    pub fn is_key_released_global(&self, key_code: KeyCode) -> bool {
        self.global_just_released.contains(&key_code)
    }

    pub fn clear(&mut self) {
        self.typed_string.clear();
        self.just_pressed.clear();
        self.just_released.clear();
        self.global_just_pressed.clear();
        self.global_just_released.clear();
    }

    pub fn poll_global(
        &mut self,
        key_events: impl Iterator<Item = (KeyCode, Option<String>, bool)>,
    ) {
        for (key, name, down) in key_events {
            if down && self.global_pressed_keys.contains(&key) {
                continue; // key repeat
            }
            self.set_key_pressed_global(key, name, down);
        }
    }
}

use rdev::Key;
use winit::keyboard::KeyCode as K;

pub fn map_key(k: Key) -> Option<K> {
    Some(match k {
        Key::KeyA => K::KeyA,
        Key::KeyB => K::KeyB,
        Key::KeyC => K::KeyC,
        Key::KeyD => K::KeyD,
        Key::KeyE => K::KeyE,
        Key::KeyF => K::KeyF,
        Key::KeyG => K::KeyG,
        Key::KeyH => K::KeyH,
        Key::KeyI => K::KeyI,
        Key::KeyJ => K::KeyJ,
        Key::KeyK => K::KeyK,
        Key::KeyL => K::KeyL,
        Key::KeyM => K::KeyM,
        Key::KeyN => K::KeyN,
        Key::KeyO => K::KeyO,
        Key::KeyP => K::KeyP,
        Key::KeyQ => K::KeyQ,
        Key::KeyR => K::KeyR,
        Key::KeyS => K::KeyS,
        Key::KeyT => K::KeyT,
        Key::KeyU => K::KeyU,
        Key::KeyV => K::KeyV,
        Key::KeyW => K::KeyW,
        Key::KeyX => K::KeyX,
        Key::KeyY => K::KeyY,
        Key::KeyZ => K::KeyZ,

        Key::Num0 => K::Digit0,
        Key::Num1 => K::Digit1,
        Key::Num2 => K::Digit2,
        Key::Num3 => K::Digit3,
        Key::Num4 => K::Digit4,
        Key::Num5 => K::Digit5,
        Key::Num6 => K::Digit6,
        Key::Num7 => K::Digit7,
        Key::Num8 => K::Digit8,
        Key::Num9 => K::Digit9,

        Key::F1 => K::F1,
        Key::F2 => K::F2,
        Key::F3 => K::F3,
        Key::F4 => K::F4,
        Key::F5 => K::F5,
        Key::F6 => K::F6,
        Key::F7 => K::F7,
        Key::F8 => K::F8,
        Key::F9 => K::F9,
        Key::F10 => K::F10,
        Key::F11 => K::F11,
        Key::F12 => K::F12,

        Key::Escape => K::Escape,
        Key::Tab => K::Tab,
        Key::Space => K::Space,
        Key::Return => K::Enter,
        Key::Backspace => K::Backspace,
        Key::Delete => K::Delete,
        Key::Insert => K::Insert,
        Key::Home => K::Home,
        Key::End => K::End,
        Key::PageUp => K::PageUp,
        Key::PageDown => K::PageDown,
        Key::UpArrow => K::ArrowUp,
        Key::DownArrow => K::ArrowDown,
        Key::LeftArrow => K::ArrowLeft,
        Key::RightArrow => K::ArrowRight,
        Key::CapsLock => K::CapsLock,
        Key::NumLock => K::NumLock,
        Key::ScrollLock => K::ScrollLock,
        Key::Pause => K::Pause,
        Key::PrintScreen => K::PrintScreen,

        Key::ShiftLeft => K::ShiftLeft,
        Key::ShiftRight => K::ShiftRight,
        Key::ControlLeft => K::ControlLeft,
        Key::ControlRight => K::ControlRight,
        Key::Alt => K::AltLeft,
        Key::AltGr => K::AltRight,
        Key::MetaLeft => K::SuperLeft,
        Key::MetaRight => K::SuperRight,

        Key::BackQuote => K::Backquote,
        Key::Minus => K::Minus,
        Key::Equal => K::Equal,
        Key::LeftBracket => K::BracketLeft,
        Key::RightBracket => K::BracketRight,
        Key::SemiColon => K::Semicolon,
        Key::Quote => K::Quote,
        Key::BackSlash => K::Backslash,
        Key::IntlBackslash => K::IntlBackslash,
        Key::Comma => K::Comma,
        Key::Dot => K::Period,
        Key::Slash => K::Slash,

        Key::Kp0 => K::Numpad0,
        Key::Kp1 => K::Numpad1,
        Key::Kp2 => K::Numpad2,
        Key::Kp3 => K::Numpad3,
        Key::Kp4 => K::Numpad4,
        Key::Kp5 => K::Numpad5,
        Key::Kp6 => K::Numpad6,
        Key::Kp7 => K::Numpad7,
        Key::Kp8 => K::Numpad8,
        Key::Kp9 => K::Numpad9,
        Key::KpReturn => K::NumpadEnter,
        Key::KpPlus => K::NumpadAdd,
        Key::KpMinus => K::NumpadSubtract,
        Key::KpMultiply => K::NumpadMultiply,
        Key::KpDivide => K::NumpadDivide,
        Key::KpDelete => K::NumpadDecimal,

        Key::Function => K::Fn,
        _ => return None,
    })
}
