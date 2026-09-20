use crate::app::Flake;
use crate::global::*;
use crate::shapes::color::Color;
use winit::keyboard::KeyCode;
mod app;
mod camera;
mod draw_state;
mod global;
mod input;
mod model;
mod shapes;
mod state;

fn main() -> anyhow::Result<()> {
    let speed = 200.0;
    let (mut x, mut y) = (0.0, 0.0);

    Flake::run(|| {
        clear_background(Color::RED);

        if is_key_down(KeyCode::KeyA) { x -= speed * get_frame_time(); }
        if is_key_down(KeyCode::KeyD) { x += speed * get_frame_time(); }
        if is_key_down(KeyCode::KeyW) { y -= speed * get_frame_time(); }
        if is_key_down(KeyCode::KeyS) { y += speed * get_frame_time(); }

        draw_rectangle(x, y, 50.0, 50.0, Color::BLACK);
    })
}
