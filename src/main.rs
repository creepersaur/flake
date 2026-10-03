use flake::prelude::*;

fn main() -> FlakeResult {
    flake::run_with_bg(BLACK, async {
        loop {
            draw_text("Hello", 0.0, 0.0, 32.0, None, RED);
            
            next_frame().await;
        }
    })
}
