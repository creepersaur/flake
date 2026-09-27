use flake::prelude::*;

fn main() -> FlakeResult {
    set_fps_capped(false);

    flake::run(|| {
        draw_rectangle(50.0, 420.0, 50.0, 50.0, LIGHT_BLACK);
        draw_rectangle(100.0, 420.0, 50.0, 50.0, DARK_GRAY);
        draw_rectangle(150.0, 420.0, 50.0, 50.0, GRAY);
        draw_rectangle(200.0, 420.0, 50.0, 50.0, LIGHT_GRAY);
        draw_rectangle(200.0, 420.0, 50.0, 50.0, WHITE);

        for y in 0..10 {
            for x in 0..10 {
                let rect = Rect::new(x as f32 * 25.0, y as f32 * 25.0, 20.0, 20.0);
                draw_rectangle_from_rect(rect, if rect.contains_point_vec2(mouse_position_vec2()) {
                    LIGHT_RED
                } else {
                    RED
                });
            }
        }
    })
}
