//! # Flake
//!
//! `flake` is a game/application framework inspired by macroquad, built on WGPU. Made to create
//! games and apps very easily.
//!
//! ## Getting Started
//!
//! In your `main.rs`, write the following:
//! 
//! ```
//! use flake::prelude::*;
//! 
//! #[flake]
//! async fn main() {
//!     loop {
//!         draw_circle(mouse_position().0, mouse_position().1, 50.0, RED);
//!         draw_text("Hello from Flake!", 10.0, 10.0, 16.0, None, WHITE);
//! 
//!         end_frame!()
//!     }
//! }
//! ```

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

#[macro_export]
macro_rules! end_frame {
    () => {next_frame().await};
}