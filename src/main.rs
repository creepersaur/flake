use flake::prelude::*;
use rand::RngExt;

#[derive(Copy, Clone, PartialEq)]
enum Cell {
    Empty,
    Snake,
    Head,
    Apple,
}

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn main() -> FlakeResult {
    const W: usize = 30;
    const H: usize = 20;
    set_window_size(W as u32 * 32, H as u32 * 32);

    let mut direction = Direction::Right;
    let mut head = (15, 10);
    let mut grid = [[Cell::Empty; W]; H];

    let mut rng = rand::rng();
    grid[rng.random_range(0..H)][rng.random_range(0..W)] = Cell::Apple;

    let mut game_over = false;
    let mut score = 0;
    let mut positions: Vec<(usize, usize)> = vec![];

    let timer = add_timer(Timer::new(0.1));

    flake::run(|| {
        if is_key_pressed(KeyCode::KeyD) {
            direction = Direction::Right
        };
        if is_key_pressed(KeyCode::KeyA) {
            direction = Direction::Left
        };
        if is_key_pressed(KeyCode::KeyW) {
            direction = Direction::Up
        };
        if is_key_pressed(KeyCode::KeyS) {
            direction = Direction::Down
        };

        timer.on_completion(|| {
            clear_background(BLACK);
            
            match direction {
                Direction::Up => head.1 -= 1,
                Direction::Down => head.1 += 1,
                Direction::Right => head.0 += 1,
                Direction::Left => head.0 -= 1,
            }

            if grid[head.1][head.0] == Cell::Apple {
                score += 1;
                grid[rng.random_range(0..H)][rng.random_range(0..W)] = Cell::Apple;
            }

            if game_over {
                flake_sleep(100000.0);
                flake_quit();
            }

            if grid[head.1][head.0] == Cell::Snake {
                draw_text("GAME OVER", 100.0, 100.0, 64.0, None, RED);
                game_over = true;
            }

            for (x, y) in positions.iter() {
                grid[*y][*x] = Cell::Empty;
            }

            positions.insert(0, head);
            positions.resize(score + 1, (0, 0));

            for (x, y) in positions.iter() {
                grid[*y][*x] = Cell::Snake;
            }

            grid[head.1][head.0] = Cell::Head;

            for y in 0..H {
                for x in 0..W {
                    let grid_cell = grid[y][x];
                    if grid_cell != Cell::Empty {
                        draw_rectangle(
                            x as f32 * 32.0,
                            y as f32 * 32.0,
                            30.0,
                            30.0,
                            match grid_cell {
                                Cell::Head => GREEN,
                                Cell::Snake => DARK_GREEN,
                                Cell::Apple => RED,
                                _ => unreachable!(),
                            },
                        );
                    }
                }
            }

            draw_text(&format!("Score: {score}"), 5.0, 5.0, 16.0, None, WHITE);
        });

        if timer.completed() {
            timer.reset();
            add_timer(timer.clone());
        }
    })
}
