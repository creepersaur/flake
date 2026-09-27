use flake::prelude::*;

fn main() -> FlakeResult {
    set_fps_capped(false);

    run! {
        draw_text(&format!("FPS: {}", get_fps()), 0.0, 0.0, 16.0, WHITE);
    }
}
