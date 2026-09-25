use flake::prelude::*;

fn main() -> FlakeResult {
    run! {
        draw_circle(0.0, 0.0, 100.0, GREEN);
        draw_circle(
            mouse_position().0,
            mouse_position().1,
            50.0,
            RED
        );
    }
}
