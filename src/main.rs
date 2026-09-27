use flake::prelude::*;

fn main() -> FlakeResult {
    let rect = Rect::new(0.0, 0.0, 50.0, 50.0);
    flake::run_with_bg(TRANSPARENT, || {
        rect.draw(RED);
        draw_circle(mouse_position().0, mouse_position().1, 20.0, WHITE);
    })
}
