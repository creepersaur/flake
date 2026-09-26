use flake::prelude::*;

fn main() -> FlakeResult {
    run! {
        set_fps_capped(false);
        draw_text(&format!("FPS: {}", get_fps()), 0.0, 0.0, 16.0, WHITE);
    }
}
