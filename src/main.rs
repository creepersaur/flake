use flake::prelude::*;

fn main() -> flake::Result {
    let speed = 200.0;
    let (mut x, mut y) = (0.0, 0.0);

    flake::run(|| {
        clear_background(Color::RED);

        if is_key_down(KeyCode::KeyA) { x -= speed * get_frame_time(); }
        if is_key_down(KeyCode::KeyD) { x += speed * get_frame_time(); }
        if is_key_down(KeyCode::KeyW) { y -= speed * get_frame_time(); }
        if is_key_down(KeyCode::KeyS) { y += speed * get_frame_time(); }

        draw_rectangle(x, y, 50.0, 50.0, Color::BLACK);

        if is_key_down(KeyCode::Space) {
            flake_quit();
        }
    })
}
