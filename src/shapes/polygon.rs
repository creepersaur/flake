#![allow(unused)]
use cgmath::Vector2;

pub fn cross(a: Vector2<f32>, b: Vector2<f32>, c: Vector2<f32>) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

pub fn point_in_tri(p: Vector2<f32>, a: Vector2<f32>, b: Vector2<f32>, c: Vector2<f32>) -> bool {
    let d1 = cross(a, b, p);
    let d2 = cross(b, c, p);
    let d3 = cross(c, a, p);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}