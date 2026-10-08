use flake::prelude::*;

#[flake]
async fn main() {
    loop {
        clear_background(BLACK);

        draw_circle(25.0, 25.0, 25.0, BLUE.with_alpha(0.5));
        
        rect(0.0, 0.0, 50.0, 50.0);

        next_frame().await
    }
}
