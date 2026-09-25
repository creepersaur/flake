use flake::prelude::*;

fn main() -> flake::Result {
    let speed = 200.0;
    let mut position = Vector2::new(390.0, 290.0);
    let mut rot = 0.0;

    flake::run(|| {
        clear_background(BLACK);

        let dir = Vector2::new(
            get_axis(KeyCode::KeyA, KeyCode::KeyD),
            get_axis(KeyCode::KeyW, KeyCode::KeyS),
        )
        .normalize_or_zero();

        if is_mouse_button_released(MouseButton::Left) {
            println!("Space clicked!")
        }

        position += dir * speed * get_frame_time();
        draw_rectangle(position.x, position.y, 20.0, 20.0, RED);

        draw_rectangle_lines_rotated(300.0, 200.0, 200.0, 200.0, rot, 5.0, GREEN);
        rot += 0.1f32.to_radians();

        if is_key_down(KeyCode::Space) {
            flake_quit();
        }
    })
}
