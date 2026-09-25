//! # Flake
//!
//! `flake` is a game/application framework inspired by macroquad, built on WGPU. Made to create
//! games and apps very easily.

use crate::app::App;
use crate::prelude::{clear_background, Color};

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

pub type FlakeResult = anyhow::Result<()>;

pub fn run(update_fn: impl FnMut()) -> anyhow::Result<()> {
    App::run(update_fn)
}

pub fn run_with_bg(color: Color, mut update_fn: impl FnMut()) -> anyhow::Result<()> {
    App::run(|| {
        clear_background(color);
        update_fn();
    })
}

#[macro_export]
macro_rules! run {
    ($color:expr => {$($tokens:tt)*}) => {
        flake::run_with_bg($color, || { $($tokens)* })
    };

    {$($tokens:tt)*} => {
        flake::run(|| { $($tokens)* })
    };
}