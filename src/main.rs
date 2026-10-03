use flake::prelude::*;

#[flake]
async fn main() {
    loop {
        clear_background(RED);
        draw_circle(75.0, 75.0, 50.0, BLACK);

        next_frame().await
    }
}