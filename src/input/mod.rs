use winit::event::MouseButton;
use winit::keyboard::KeyCode;

pub mod keyboard;
pub(crate) mod mouse;

pub enum GlobalInputEvent {
    Button(MouseButton, bool), // (button, is_pressed)
    Wheel(f32, f32),
    Move(f32, f32),
    Key(KeyCode, bool), // (key, is_pressed)
}