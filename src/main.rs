use flake::prelude::*;

const GRAVITY: f32 = 900.0;
const RESTITUTION: f32 = 0.5;
const SUBSTEPS: usize = 4;

struct Block {
    rect: Rect,
    angle: f32,
}

struct Ball {
    pos: Vector2,
    vel: Vector2,
    r: f32,
}

/// Circle vs OBB. Returns (normal pointing from box to circle, penetration depth).
fn circle_vs_obb(c: Vector2, r: f32, b: &Rect, angle: f32) -> Option<(Vector2, f32)> {
    let (s, co) = angle.sin_cos();
    let d = c - b.center();
    // into the box's local space
    let local = vec2(d.x * co + d.y * s, -d.x * s + d.y * co);
    let h = b.size() * 0.5;

    let closest = vec2(local.x.clamp(-h.x, h.x), local.y.clamp(-h.y, h.y));
    let diff = local - closest;
    let dist2 = diff.x * diff.x + diff.y * diff.y;

    let (n_local, pen) = if dist2 > 1e-8 {
        if dist2 >= r * r {
            return None;
        }
        let dist = dist2.sqrt();
        (diff / dist, r - dist)
    } else {
        // center is inside the box: push out through the nearest face
        let dx = h.x - local.x.abs();
        let dy = h.y - local.y.abs();
        if dx < dy {
            (vec2(if local.x >= 0.0 { 1.0 } else { -1.0 }, 0.0), dx + r)
        } else {
            (vec2(0.0, if local.y >= 0.0 { 1.0 } else { -1.0 }), dy + r)
        }
    };

    // back to world space
    let n = vec2(n_local.x * co - n_local.y * s, n_local.x * s + n_local.y * co);
    Some((n, pen))
}

#[flake]
async fn main() {
    let mut ground = vec![
        Block { rect: rect(0.0, 500.0, 800.0, 100.0), angle: 0.0 },
        Block { rect: rect(300.0, 430.0, 200.0, 20.0), angle: -0.4 },
    ];
    let mut balls: Vec<Ball> = Vec::new();
    let mut spawned = 0;

    let mut drag: Option<Vector2> = None;
    let mut drag_angle = 0.0_f32;
    let mut head: Vec<i32> = vec![-1; 64 * 48];
    let mut next: Vec<i32> = Vec::new();

    loop {
        clear_background(BLACK);
        let dt = get_frame_time().min(1.0 / 30.0);

        draw_text(&format!("FPS: {}", get_fps()), 10.0, 10.0, 16.0, None, WHITE);
        draw_text(&format!("balls: {spawned}"), 10.0, 26.0, 16.0, None, WHITE);

        // SPAWN (hold right click)
        for _ in 0..20 {
            if is_mouse_button_down(MouseButton::Right) {
                let p = mouse_position_vec2();
                let r = 8.0 + (spawned % 5) as f32 * 4.0;
                // tiny x offset so stacked balls don't spawn perfectly aligned
                let jitter = (spawned % 7) as f32 - 3.0;
                balls.push(Ball { pos: p + vec2(jitter, 0.0), vel: vec2(0.0, 0.0), r: r * 0.3 });
                spawned += 1;
            }
        }

        // PHYSICS
        let sdt = dt / SUBSTEPS as f32;
        for _ in 0..SUBSTEPS {
            for b in balls.iter_mut() {
                b.vel.y += GRAVITY * sdt;
                b.pos += b.vel * sdt;
            }

            // balls vs blocks
            for b in balls.iter_mut() {
                for block in &ground {
                    if let Some((n, pen)) = circle_vs_obb(b.pos, b.r, &block.rect, block.angle) {
                        b.pos += n * pen;
                        let vn = b.vel.dot(n);
                        if vn < 0.0 {
                            b.vel -= n * (vn * (1.0 + RESTITUTION));
                        }
                    }
                }
            }

            // balls vs balls (uniform grid)
            const CELL: f32 = 48.0; // >= largest ball diameter (max r is 24)
            const GW: usize = 64;
            const GH: usize = 48;

            let cell_of = |p: Vector2| -> (i32, i32) {
                (
                    ((p.x / CELL) as i32).clamp(0, GW as i32 - 1),
                    ((p.y / CELL) as i32).clamp(0, GH as i32 - 1),
                )
            };

            head.fill(-1);
            next.resize(balls.len(), -1);
            for (i, b) in balls.iter().enumerate() {
                let (cx, cy) = cell_of(b.pos);
                let c = cy as usize * GW + cx as usize;
                next[i] = head[c];
                head[c] = i as i32;
            }

            for i in 0..balls.len() {
                let (cx, cy) = cell_of(balls[i].pos);
                for ny in (cy - 1).max(0)..=(cy + 1).min(GH as i32 - 1) {
                    for nx in (cx - 1).max(0)..=(cx + 1).min(GW as i32 - 1) {
                        let mut j = head[ny as usize * GW + nx as usize];
                        while j >= 0 {
                            let ju = j as usize;
                            if ju > i {
                                let d = balls[ju].pos - balls[i].pos;
                                let dist = (d.x * d.x + d.y * d.y).sqrt();
                                let min = balls[i].r + balls[ju].r;
                                if dist < min && dist > 1e-6 {
                                    let n = d / dist;
                                    let pen = min - dist;
                                    let (mi, mj) = (balls[i].r * balls[i].r, balls[ju].r * balls[ju].r);
                                    let total = mi + mj;

                                    balls[i].pos -= n * (pen * mj / total);
                                    balls[ju].pos += n * (pen * mi / total);

                                    let vn = (balls[ju].vel - balls[i].vel).dot(n);
                                    if vn < 0.0 {
                                        let imp = -(1.0 + RESTITUTION) * vn / (1.0 / mi + 1.0 / mj);
                                        balls[i].vel -= n * (imp / mi);
                                        balls[ju].vel += n * (imp / mj);
                                    }
                                }
                            }
                            j = next[ju];
                        }
                    }
                }
            }
        }

        // drop balls that fell off the world
        balls.retain(|b| b.pos.y < 2000.0);

        // BLOCK DRAGGING (Q / E rotate)
        if is_mouse_button_pressed(MouseButton::Left) {
            drag = Some(mouse_position_vec2());
            drag_angle = 0.0;
        }
        if drag.is_some() {
            drag_angle += 2.0 * dt * get_axis(KeyCode::KeyQ, KeyCode::KeyE);
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Some(start) = drag {
                ground.push(Block {
                    rect: Rect::from_corners(start, mouse_position_vec2()),
                    angle: drag_angle,
                });
                drag = None;
            }
        }

        // DRAWING
        for b in &balls {
            draw_circle(b.pos.x, b.pos.y, b.r, RED);
        }
        for block in &ground {
            block.rect.draw_lines_rotated(block.angle, 1.0, GREEN);
            block.rect.draw_rotated(block.angle, GREEN.with_alpha(0.5));
        }
        if let Some(start) = drag {
            let preview = Rect::from_corners(start, mouse_position_vec2());
            preview.draw_lines_rotated(drag_angle, 1.0, LIGHT_BLUE);
            preview.draw_rotated(drag_angle, BLUE.with_alpha(0.05));
        }

        next_frame().await
    }
}