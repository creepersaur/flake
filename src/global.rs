#![allow(unused)]
use crate::model::instance::Instance;
use crate::shapes::Shape;
use crate::shapes::color::Color;
use crate::state::State;
use cgmath::{Quaternion, Vector2, Vector3};
use std::cell::Cell;
use std::ptr::NonNull;

thread_local! {
    pub static STATE: Cell<Option<NonNull<State>>> = const { Cell::new(None) };
}

fn with_global<R>(state: &mut State, f: impl FnOnce() -> R) -> R {
    STATE.set(Some(NonNull::from(&mut *state)));
    let r = f();
    STATE.set(None);
    r
}

fn ctx<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| {
        let mut p = s
            .get()
            .expect("no active context (call only inside the update fn)");
        f(unsafe { p.as_mut() })
    })
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// # Out-facing API

pub fn mouse_position() -> (f32, f32) {
    ctx(|s| s.mouse_state.get_position().into())
}
pub fn get_fps() -> f32 {
    ctx(|s| s.get_fps())
}

/// ## DRAWING

pub fn clear_background(c: Color) {
    ctx(|s| {
        s.clear_color = c;
        s.draw_state.clear();
    });
}

pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, c: Color) {
    ctx(|s| s.draw_state.draw_rectangle(x, y, w, h, c));
}
pub fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32, thickness: f32, c: Color) {
    ctx(|s| s.draw_state.draw_rectangle_lines(x, y, w, h, thickness, c));
}

pub fn draw_rectangle_rotated(x: f32, y: f32, w: f32, h: f32, rotation: f32, c: Color) {
    ctx(|s| s.draw_state.draw_rectangle_rotated(x, y, w, h, rotation, c));
}
pub fn draw_rectangle_lines_rotated(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    thickness: f32,
    c: Color,
) {
    ctx(|s| {
        s.draw_state
            .draw_rectangle_lines_rotated(x, y, w, h, rotation, thickness, c)
    });
}

pub fn draw_circle(x: f32, y: f32, r: f32, c: Color) {
    ctx(|s| s.draw_state.draw_circle(x, y, r, c));
}
pub fn draw_circle_lines(x: f32, y: f32, r: f32, thickness: f32, c: Color) {
    ctx(|s| s.draw_state.draw_circle_lines(x, y, r, thickness, c));
}

pub fn draw_triangle(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, color: Color) {
    ctx(|s| s.draw_state.draw_triangle(x1, y1, x2, y2, x3, y3, color));
}
pub fn draw_triangle_lines(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    thickness: f32,
    color: Color,
) {
    ctx(|s| {
        s.draw_state
            .draw_triangle_lines(x1, y1, x2, y2, x3, y3, thickness, color)
    });
}

pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    ctx(|s| s.draw_state.draw_line(x1, y1, x2, y2, thickness, color))
}

pub fn draw_poly_line(points: &[Vector2<f32>], thickness: f32, color: Color, closed: bool) {
    ctx(|s| {
        s.draw_state
            .draw_poly_line_miter(points, thickness, color, closed)
    })
}

pub fn draw_poly_line_rounded(points: &[Vector2<f32>], thickness: f32, color: Color, closed: bool) {
    ctx(|s| {
        s.draw_state
            .draw_poly_line(points, thickness, color, closed, true)
    })
}

pub fn draw_polygon(points: &[Vector2<f32>], thickness: f32, color: Color) {
    ctx(|s| s.draw_state.draw_polygon(points, color))
}
pub fn draw_polygon_concave(points: &[Vector2<f32>], thickness: f32, color: Color) {
    ctx(|s| s.draw_state.draw_polygon_concave(points, color))
}

pub fn draw_text(t: &str, x: f32, y: f32, size: f32, c: Color) {
    ctx(|s| s.draw_state.draw_text(t, x, y, size, c));
}
