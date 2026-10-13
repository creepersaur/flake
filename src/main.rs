use flake::prelude::*;

#[flake(skip_taskbar = true)]
async fn main() {
    let screensize = screen_size();
    set_window_size(screensize.x as u32 - 1, screensize.y as u32 - 1);
    set_window_position(0, 0);
    set_window_decorations(false);
    set_window_passthrough(true);
    set_window_level(WindowLevel::AlwaysOnTop);

    let (mut px, mut py) = (200.0_f32, 200.0_f32);
    let mut t = 0.0_f32;
    let mut walk = 0.0_f32; // 0 = idle, 1 = full walk (eased)
    let mut facing = 1.0_f32; // smoothed: +1 right, -1 left
    let mut face_target = 1.0_f32;
    let mut tilt = 0.0_f32;

    loop {
        clear_background(TRANSPARENT);
        let dt = get_frame_time();

        let (mx, my) = mouse_screen_position();
        let (mx, my) = (mx as f32, my as f32);

        let (dx, dy) = (mx - px, my - py);
        let dist = (dx * dx + dy * dy).sqrt().max(0.001);
        let (nx, ny) = (dx / dist, dy / dist);

        // Ease toward the current direction (frame-rate independent)
        if nx.abs() > 0.2 {
            face_target = nx.signum();
        }
        facing += (face_target - facing) * (1.0 - (-15.0 * dt).exp());

        let (tx, ty) = (mx - nx * 80.0, my - ny * 80.0);
        let moving = dist > 90.0;
        let speed = ((dist - 90.0) / 510.0).clamp(0.0, 1.0);

        if moving {
            let k = 1.0 - (-5.0 * dt).exp();
            px += (tx - px) * k;
            py += (ty - py) * k;
            t += dt * (10.0 + speed * 20.0);
        }

        // Ease walk amount in/out (frame-rate independent)
        let target = if moving { 1.0 } else { 0.0 };
        walk += (target - walk) * (1.0 - (-12.0 * dt).exp());

        // Body: bounces once per step (twice per cycle), highest when legs are under it
        let bob = -t.cos().abs() * 6.0 * walk;
        let (x, y0) = (px - 16.0, py - 16.0);
        let y = y0 + bob;

        // Feet: opposite phases (PI apart). Feet stay on the ground, not on the bob.
        let stride = (4.0 + speed * 3.0) * walk * facing;
        let lift_h = (3.0 + speed * 2.0) * walk;
        for (fx, phase) in [(x + 4.0, 0.0), (x + 20.0, std::f32::consts::PI)] {
            let p = t + phase;
            let off = p.sin() * stride;               // slide back/forward
            let lift = p.cos().max(0.0) * lift_h;     // lifted only while swinging forward
            draw_rectangle(fx + off, y0 + 30.0 - lift, 8.0, 8.0, RED.with_alpha(0.7));
            draw_rectangle_lines(fx + off, y0 + 30.0 - lift, 8.0, 8.0, 2.0, BLACK);
        }

        // lean toward the mouse horizontally, max ~11 degrees
        let tilt_target = nx * 0.05;
        tilt += (tilt_target - tilt) * (1.0 - (-10.0 * dt).exp());

        // rotated shapes use their center as (x, y)
        let (cx, cy) = (x + 16.0, y + 16.0);
        let (s, c) = tilt.sin_cos();
        let rot = |ox: f32, oy: f32| (cx + ox * c - oy * s, cy + ox * s + oy * c);

        draw_rectangle_rotated(cx, cy, 32.0, 32.0, tilt, RED.with_alpha(0.7));
        draw_rectangle_lines_rotated(cx, cy, 32.0, 32.0, tilt, 2.0, BLACK);
        draw_rectangle_rotated(cx, cy + 8.0, 8.0, 2.0, tilt, BLACK);
        draw_rectangle_rotated(cx - 4.0, cy + 8.0, 2.0, 4.0, tilt, BLACK);
        draw_rectangle_rotated(cx + 4.0, cy + 8.0, 2.0, 4.0, tilt, BLACK);

        // eye centers relative to the body center
        for eox in [-6.5, 7.5] {
            let (ex, ey) = rot(eox, -4.5);
            draw_rectangle_rotated(ex, ey, 7.0, 7.0, tilt, WHITE);
            draw_rectangle_rotated(ex + nx * 2.0, ey + ny * 2.0, 3.0, 3.0, tilt, BLACK);
        }

        next_frame().await
    }
}