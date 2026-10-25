use flake::prelude::*;

#[flake]
async fn main() {
    let target = render_target(100, 100, FilterType::Linear);
    let render_cam = Camera2D {
        render_target: Some(target.clone()),
        ..Default::default()
    };

    loop {
        clear_background(BLACK);

        set_camera(&render_cam);

        draw_circle(mouse_position().0, mouse_position().1, 50.0, RED);

        set_default_camera();

        draw_texture(target.texture, 0.0, 0.0, 100.0, 100.0, WHITE);

        end_frame!();
    }
}
