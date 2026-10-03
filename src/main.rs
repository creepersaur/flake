use flake::prelude::*;

#[flake]
async fn main() {
    let mut player = Rect::new(0.0, 0.0, 50.0, 50.0);

    loop {
        clear_background(BLACK);

        player = player.with_position(
            player.position()
                + get_vector(KeyCode::KeyA, KeyCode::KeyD, KeyCode::KeyW, KeyCode::KeyS)
                    * 500.0
                    * get_frame_time(),
        );

        draw_rectangle_from_rect(player, RED);

        next_frame().await;
    }
}
