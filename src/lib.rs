//! # Flake
//!
//! `flake` is a game/application framework inspired by macroquad, built on WGPU. Made to create
//! games and apps very easily.

use crate::app::App;

mod app;
mod camera;
mod draw_state;
pub mod prelude;
mod input;
mod model;
mod shapes;
mod state;
mod global;
mod misc;

pub type Result = anyhow::Result<()>;

pub fn run(update_fn: impl FnMut()) -> anyhow::Result<()> {
    App::run(update_fn)
}