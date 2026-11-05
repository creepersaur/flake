use flake::prelude::*;

#[flake]
async fn main() {
    set_window_decorations(false);
    set_window_level(WindowLevel::AlwaysOnTop);

    block_key(KeyCode::KeyH);
    
    loop {
        clear_background(WHITE);

        if is_key_down_global(KeyCode::KeyH) {
            draw_circle(0.0, 0.0, 50.0, RED);
        }
        
        end_frame!();
    }
}