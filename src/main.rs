use flake::prelude::*;

fn main() -> FlakeResult {
    let mut rect = Rect::new(0.0, 0.0, 50.0, 50.0);
    let speed = 50.0;

    flake::run(|| {
        rect.x += speed * get_frame_time();

        rect.draw(RED);
    })
}
