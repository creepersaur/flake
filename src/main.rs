use flake::prelude::*;

fn main() -> FlakeResult {
    const DIST: f32 = 50.0;

    run! {
        let mut p = Vector2::new(100.0, 100.0);

        for i in 0..10 {
            let dir = (i as f32 * mouse_position().0).to_radians();
            let new_point = Vector2::new(p.x + dir.cos() * DIST, p.y + dir.sin() * DIST);
            draw_arrow(p.x, p.y, new_point.x, new_point.y, 5.0, 20.0, RED);

            p = new_point;
        }
    }
}
