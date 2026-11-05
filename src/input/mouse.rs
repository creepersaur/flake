use crate::input::GlobalInputEvent;
use crate::prelude::vec2;
use cgmath::{Vector2, prelude::*};
use rdev::Button;
use std::collections::HashSet;
use std::vec::IntoIter;
use winit::event::MouseButton;
use winit::window::Window;

pub(crate) fn map_button(b: Button) -> MouseButton {
    match b {
        Button::Left => MouseButton::Left,
        Button::Right => MouseButton::Right,
        Button::Middle => MouseButton::Middle,
        Button::Unknown(n) => MouseButton::Other(n as u16),
    }
}

#[derive(Clone, PartialEq)]
pub struct MouseState {
    delta: Option<Vector2<f32>>,
    position: Vector2<f32>,
    screen_position: Vector2<f32>,
    pressed_buttons: HashSet<MouseButton>,
    mouse_wheel: Vector2<f32>,
    just_pressed: HashSet<MouseButton>,
    just_released: HashSet<MouseButton>,

    global_delta: Option<Vector2<f32>>,
    global_position: Vector2<f32>,
    global_screen_position: Vector2<f32>,
    global_pressed_buttons: HashSet<MouseButton>,
    global_mouse_wheel: Vector2<f32>,
    global_just_pressed: HashSet<MouseButton>,
    global_just_released: HashSet<MouseButton>,
}

impl MouseState {
    pub fn default() -> Self {
        Self {
            delta: None,
            screen_position: Vector2::zero(),
            position: Vector2::zero(),
            pressed_buttons: Default::default(),
            mouse_wheel: Vector2::zero(),
            just_pressed: Default::default(),
            just_released: Default::default(),

            global_delta: None,
            global_position: Vector2::zero(),
            global_screen_position: Vector2::zero(),
            global_pressed_buttons: Default::default(),
            global_mouse_wheel: Vector2::zero(),
            global_just_pressed: Default::default(),
            global_just_released: Default::default(),
        }
    }

    pub fn set_pressed_button(&mut self, button: MouseButton, is_pressed: bool) {
        match is_pressed {
            true => {
                self.pressed_buttons.insert(button);
                self.just_pressed.insert(button)
            }
            false => {
                self.pressed_buttons.remove(&button);
                self.just_released.insert(button)
            }
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
    pub fn is_button_down_global(&self, button: MouseButton) -> bool {
        self.global_pressed_buttons.contains(&button)
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn is_button_pressed_global(&self, button: MouseButton) -> bool {
        self.global_just_pressed.contains(&button)
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn is_button_released_global(&self, button: MouseButton) -> bool {
        self.global_just_released.contains(&button)
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn clear_just_pressed(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();

        self.global_just_pressed.clear();
        self.global_just_released.clear();
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn clear_mouse_wheel(&mut self) {
        self.mouse_wheel.set_zero();
        self.global_mouse_wheel.set_zero();
    }

    pub fn clear_delta(&mut self) {
        if let Some(d) = &mut self.delta { d.set_zero(); }
        if let Some(d) = &mut self.global_delta { d.set_zero(); }
    }
    
    pub fn clear(&mut self) {
        self.clear_delta();
        self.clear_just_pressed();
        self.clear_mouse_wheel();
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

    pub fn set_screen_position(&mut self, x: f32, y: f32) {
        self.screen_position = Vector2::new(x, y);
    }

    pub fn set_position_global(&mut self, x: f32, y: f32) {
        let new_pos = Vector2::new(x, y);
        if let Some(delta) = &mut self.global_delta {
            *delta = new_pos - self.global_position;
        } else {
            self.global_delta = Some(Vector2::zero());
        }
        self.global_position = new_pos;
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn get_position_global(&self) -> Vector2<f32> {
        self.global_position
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn get_screen_position_global(&self) -> Vector2<f32> {
        self.global_screen_position
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

    pub fn set_pressed_button_global(&mut self, button: MouseButton, is_pressed: bool) {
        if is_pressed {
            self.global_pressed_buttons.insert(button);
            self.global_just_pressed.insert(button);
        } else {
            self.global_pressed_buttons.remove(&button);
            self.global_just_released.insert(button);
        }
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn accumulate_mouse_wheel_global(&mut self, x: f32, y: f32) {
        self.global_mouse_wheel += vec2(x, y);
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn get_mouse_wheel_global(&self) -> Vector2<f32> {
        self.global_mouse_wheel
    }

    pub fn poll_global(&mut self, rx: IntoIter<GlobalInputEvent>, window: &Window) {
        for ev in rx {
            match ev {
                GlobalInputEvent::Button(b, down) => self.set_pressed_button_global(b, down),
                GlobalInputEvent::Wheel(x, y) => self.accumulate_mouse_wheel_global(x, y),
                GlobalInputEvent::Move(x, y) => {
                    self.global_screen_position = Vector2::new(x, y);
                    if let Ok(p) = window.inner_position() {
                        self.set_position_global(x - p.x as f32, y - p.y as f32);
                    }
                }

                _ => {}
            }
        }
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn accumulate_mouse_wheel(&mut self, x: f32, y: f32) {
        self.mouse_wheel += vec2(x, y);
    }
}
