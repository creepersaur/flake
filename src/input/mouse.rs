use cgmath::{Vector2, prelude::*};
use device_query::{DeviceQuery, DeviceState};
use std::collections::HashSet;
use winit::event::MouseButton;
use winit::window::Window;

#[derive(Clone, PartialEq)]
pub struct MouseState {
    position: Vector2<f32>,
    delta: Option<Vector2<f32>>,
    pressed_buttons: HashSet<MouseButton>,
}

impl MouseState {
    pub fn default() -> Self {
        Self {
            position: Vector2::zero(),
            delta: None,
            pressed_buttons: Default::default(),
        }
    }

    pub fn set_pressed_button(&mut self, button: MouseButton, is_pressed: bool) {
        match is_pressed {
            true => self.pressed_buttons.insert(button),
            false => self.pressed_buttons.remove(&button),
        };
    }

    #[allow(unused)]
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        let new_pos: Vector2<f32> = Vector2::new(x, y);
        if let Some(delta) = &mut self.delta {
            *delta = new_pos - self.position;
        } else {
            self.delta = Some(Vector2::zero())
        }
        self.position = new_pos;
    }

    pub fn get_position(&self) -> Vector2<f32> {
        self.position
    }

    pub fn update_position(&mut self, window: &Window, device_state: &DeviceState) {
        if let Ok(inner_position) = window.inner_position() {
            let mouse = device_state.get_mouse();
            self.set_position(
                mouse.coords.0 as f32 - inner_position.x as f32,
                mouse.coords.1 as f32 - inner_position.y as f32,
            );
        }
    }
}
