use flake::prelude::*;

#[flake]
async fn main() {
    loop {
        clear_background(BLACK);

        draw_rectangle(0.0, 0.0, 50.0, 50.0, BLUE);
        draw_circle(50.0, 50.0, 50.0, RED);

        next_frame().await;
    }
}
