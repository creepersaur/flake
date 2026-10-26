use cgmath::{Vector2, prelude::*};
use device_query::{DeviceQuery, DeviceState};
use std::collections::HashSet;
use winit::event::MouseButton;
use winit::window::Window;
use crate::prelude::vec2;

#[derive(Clone, PartialEq)]
pub struct MouseState {
    screen_position: Vector2<f32>,
    position: Vector2<f32>,
    mouse_wheel: Vector2<f32>,
    delta: Option<Vector2<f32>>,
    pressed_buttons: HashSet<MouseButton>,
    just_pressed: HashSet<MouseButton>,
    just_released: HashSet<MouseButton>,
}

impl MouseState {
    pub fn default() -> Self {
        Self {
            screen_position: Vector2::zero(),
            position: Vector2::zero(),
            mouse_wheel: Vector2::zero(),
            delta: None,
            pressed_buttons: Default::default(),
            just_pressed: Default::default(),
            just_released: Default::default(),
        }
    }

    pub fn set_pressed_button(&mut self, button: MouseButton, is_pressed: bool) {
        match is_pressed {
            true => {
                self.pressed_buttons.insert(button);
                self.just_pressed.insert(button)
            },
            false => {
                self.pressed_buttons.remove(&button);
                self.just_released.insert(button)
            },
        };
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn is_button_down(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed.contains(&button)
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn is_button_released(&self, button: MouseButton) -> bool {
        self.just_released.contains(&button)
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn clear_just_pressed(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn clear_mouse_wheel(&mut self) {
        self.mouse_wheel.set_zero();
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

    #[allow(unused)]
    #[inline(always)]
    pub fn get_position(&self) -> Vector2<f32> {
        self.position
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn get_mouse_wheel(&self) -> Vector2<f32> {
        self.mouse_wheel
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn get_screen_position(&self) -> Vector2<f32> {
        self.screen_position
    }

    pub fn update_position(&mut self, window: &Window, device_state: &DeviceState) {
        if let Ok(inner_position) = window.inner_position() {
            let mouse = device_state.get_mouse();
            
            self.set_position(
                mouse.coords.0 as f32 - inner_position.x as f32,
                mouse.coords.1 as f32 - inner_position.y as f32,
            );

            self.screen_position = Vector2::new(mouse.coords.0 as f32, mouse.coords.1 as f32);
        }
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn set_mouse_wheel(&mut self, x: f32, y: f32) {
        self.mouse_wheel = vec2(x, y);
    }
}
