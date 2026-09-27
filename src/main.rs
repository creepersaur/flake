use flake::prelude::*;

const COLS: usize = 10;
const ROWS: usize = 20;
const CELL: f32 = 20.0;

type Grid = [[Option<Color>; COLS]; ROWS];

const SHAPES: [[[u8; 4]; 4]; 7] = [
    [[0,0,0,0],[1,1,1,1],[0,0,0,0],[0,0,0,0]], // I
    [[1,1,0,0],[1,1,0,0],[0,0,0,0],[0,0,0,0]], // O
    [[0,1,0,0],[1,1,1,0],[0,0,0,0],[0,0,0,0]], // T
    [[0,1,1,0],[1,1,0,0],[0,0,0,0],[0,0,0,0]], // S
    [[1,1,0,0],[0,1,1,0],[0,0,0,0],[0,0,0,0]], // Z
    [[1,0,0,0],[1,1,1,0],[0,0,0,0],[0,0,0,0]], // J
    [[0,0,1,0],[1,1,1,0],[0,0,0,0],[0,0,0,0]], // L
];

const COLORS: [Color; 7] = [CYAN, YELLOW, PURPLE, GREEN, RED, BLUE, ORANGE];

struct Piece {
    shape_idx: usize,
    cells: [[u8; 4]; 4],
    x: i32,
    y: i32,
}

impl Piece {
    fn new(idx: usize) -> Self {
        Self { shape_idx: idx, cells: SHAPES[idx], x: 3, y: 0 }
    }

    fn rotated(&self) -> [[u8; 4]; 4] {
        let mut new = [[0u8; 4]; 4];
        for r in 0..4 {
            for c in 0..4 {
                new[c][3 - r] = self.cells[r][c];
            }
        }
        new
    }
}

fn fits(grid: &Grid, cells: &[[u8; 4]; 4], px: i32, py: i32) -> bool {
    for r in 0..4 {
        for c in 0..4 {
            if cells[r][c] == 0 { continue; }
            let gx = px + c as i32;
            let gy = py + r as i32;
            if gx < 0 || gx >= COLS as i32 || gy >= ROWS as i32 { return false; }
            if gy >= 0 && grid[gy as usize][gx as usize].is_some() { return false; }
        }
    }
    true
}

fn lock_piece(grid: &mut Grid, piece: &Piece) {
    for r in 0..4 {
        for c in 0..4 {
            if piece.cells[r][c] == 0 { continue; }
            let gx = piece.x + c as i32;
            let gy = piece.y + r as i32;
            if gy >= 0 {
                grid[gy as usize][gx as usize] = Some(COLORS[piece.shape_idx]);
            }
        }
    }
}

fn clear_lines(grid: &mut Grid) -> u32 {
    let mut cleared = 0;
    let mut new_grid: Grid = [[None; COLS]; ROWS];
    let mut write_row = ROWS as i32 - 1;

    for r in (0..ROWS).rev() {
        if grid[r].iter().all(|c| c.is_some()) {
            cleared += 1;
        } else {
            new_grid[write_row as usize] = grid[r];
            write_row -= 1;
        }
    }
    *grid = new_grid;
    cleared
}

fn rand_shape() -> usize {
    (rand::random::<f32>() * 7.0) as usize % 7
}

fn spawn(grid: &Grid) -> Option<Piece> {
    let idx = rand_shape();
    let piece = Piece::new(idx);
    if fits(grid, &piece.cells, piece.x, piece.y) {
        Some(piece)
    } else {
        None
    }
}

struct GameState {
    grid: Grid,
    current: Option<Piece>,
    drop_timer: f32,
    drop_interval: f32,
    score: u32,
    lines: u32,
    game_over: bool,
}

impl GameState {
    fn new() -> Self {
        let grid = [[None; COLS]; ROWS];
        let current = spawn(&grid);
        Self {
            grid,
            current,
            drop_timer: 0.0,
            drop_interval: 0.5,
            score: 0,
            lines: 0,
            game_over: false,
        }
    }

    fn reset(&mut self) {
        *self = GameState::new();
    }

    fn update(&mut self, dt: f32) {
        if self.game_over {
            return;
        }

        if let Some(piece) = self.current.as_mut() {
            if is_key_pressed(KeyCode::ArrowLeft) && fits(&self.grid, &piece.cells, piece.x - 1, piece.y) {
                piece.x -= 1;
            }
            if is_key_pressed(KeyCode::ArrowRight) && fits(&self.grid, &piece.cells, piece.x + 1, piece.y) {
                piece.x += 1;
            }
            if is_key_pressed(KeyCode::ArrowUp) {
                let rotated = piece.rotated();
                if fits(&self.grid, &rotated, piece.x, piece.y) {
                    piece.cells = rotated;
                }
            }
            let soft_drop = is_key_down(KeyCode::ArrowDown);
            if is_key_pressed(KeyCode::Space) {
                while fits(&self.grid, &piece.cells, piece.x, piece.y + 1) {
                    piece.y += 1;
                }
            }

            self.drop_timer += dt;
            let interval = if soft_drop { self.drop_interval * 0.1 } else { self.drop_interval };
            if self.drop_timer >= interval {
                self.drop_timer = 0.0;
                if fits(&self.grid, &piece.cells, piece.x, piece.y + 1) {
                    piece.y += 1;
                } else {
                    lock_piece(&mut self.grid, piece);
                    let cleared = clear_lines(&mut self.grid);
                    if cleared > 0 {
                        self.lines += cleared;
                        self.score += match cleared {
                            1 => 100,
                            2 => 300,
                            3 => 500,
                            _ => 800,
                        };
                        self.drop_interval = (0.5 - (self.lines as f32 * 0.02)).max(0.1);
                    }
                    self.current = spawn(&self.grid);
                    if self.current.is_none() {
                        self.game_over = true;
                    }
                }
            }
        }
    }

    fn draw(&self) {
        clear_background(BLACK);

        if self.game_over {
            draw_text("GAME OVER", 20.0, 150.0, 20.0, RED);
            draw_text(&format!("Score: {}", self.score), 20.0, 180.0, 16.0, WHITE);
            draw_text("Press R to restart", 20.0, 210.0, 14.0, GRAY);
            return;
        }

        draw_rectangle(0.0, 0.0, COLS as f32 * CELL, ROWS as f32 * CELL, LIGHT_BLACK);

        for r in 0..ROWS {
            for c in 0..COLS {
                if let Some(color) = self.grid[r][c] {
                    draw_rectangle(c as f32 * CELL, r as f32 * CELL, CELL - 1.0, CELL - 1.0, color);
                }
            }
        }

        if let Some(piece) = self.current.as_ref() {
            let color = COLORS[piece.shape_idx];
            for r in 0..4 {
                for c in 0..4 {
                    if piece.cells[r][c] == 0 { continue; }
                    let gx = piece.x + c as i32;
                    let gy = piece.y + r as i32;
                    if gy >= 0 {
                        draw_rectangle(gx as f32 * CELL, gy as f32 * CELL, CELL - 1.0, CELL - 1.0, color);
                    }
                }
            }
        }

        let ui_x = COLS as f32 * CELL + 10.0;
        draw_text("TETRIS", ui_x, 10.0, 18.0, WHITE);
        draw_text(&format!("Score: {}", self.score), ui_x, 40.0, 14.0, WHITE);
        draw_text(&format!("Lines: {}", self.lines), ui_x, 60.0, 14.0, WHITE);
        draw_text("< > move", ui_x, 100.0, 12.0, GRAY);
        draw_text("Up: rotate", ui_x, 116.0, 12.0, GRAY);
        draw_text("Down: soft drop", ui_x, 132.0, 12.0, GRAY);
        draw_text("Space: hard drop", ui_x, 148.0, 12.0, GRAY);
    }
}

fn main() -> FlakeResult {
    set_window_size((COLS as u32) * CELL as u32 + 150, (ROWS as u32) * CELL as u32);
    set_window_title("Tetris");

    let mut game = GameState::new();

    flake::run(move || {
        if game.game_over && is_key_pressed(KeyCode::KeyR) {
            game.reset();
        }

        game.update(get_frame_time());
        game.draw();
    })
}