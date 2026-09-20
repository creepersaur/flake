use crate::app::Flake;
use crate::global::*;
use crate::shapes::color::Color;

mod app;
mod camera;
mod draw_state;
mod global;
mod input;
mod model;
mod shapes;
mod state;

fn main() -> anyhow::Result<()> {
    Flake::run(|| {
        clear_background(Color::MAGENTA);
    })
}
