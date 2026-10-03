//! # Flake
//!
//! `flake` is a game/application framework inspired by macroquad, built on WGPU. Made to create
//! games and apps very easily.

use crate::app::App;
use crate::prelude::{clear_background, Color};
pub use flake_macros::main as flake;

mod app;
mod camera;
mod draw_state;
pub mod prelude;
mod input;
mod model;
mod state;
mod global;
mod misc;
mod runtime;

pub type FlakeResult = anyhow::Result<()>;

pub fn run_async(fut: impl Future<Output = ()> + 'static) -> FlakeResult {
    App::run(fut)?;
    Ok(())
}

pub fn run_with_bg(color: Color, fut: impl Future<Output = ()> + 'static) -> FlakeResult {
    clear_background(color);
    run_async(fut)
}