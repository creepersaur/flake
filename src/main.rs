use flake::prelude::*;

struct ScrollFrame {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    content_h: f32,
    scroll: f32,
    cam: Camera2D,
    drag_from: Option<f32>,
}

impl ScrollFrame {
    fn new(x: f32, y: f32, w: f32, h: f32, content_h: f32) -> Self {
        let target = render_target(w as u32, h as u32, FilterType::Nearest);
        Self {
            x, y, w, h, content_h,
            scroll: 0.0,
            cam: Camera2D { render_target: Some(target), ..Default::default() },
            drag_from: None,
        }
    }

    fn hovered(&self) -> bool {
        let (mx, my) = mouse_position();
        mx >= self.x && mx <= self.x + self.w && my >= self.y && my <= self.y + self.h
    }

    /// Mouse position in content coordinates (what you draw with between begin/end)
    fn local_mouse(&self) -> (f32, f32) {
        let (mx, my) = mouse_position();
        (mx - self.x, my - self.y + self.scroll)
    }

    fn update(&mut self) {
        let (_, my) = mouse_position();

        // Drag to scroll
        if is_mouse_button_pressed(MouseButton::Left) && self.hovered() {
            self.drag_from = Some(my);
        }
        if !is_mouse_button_down(MouseButton::Left) {
            self.drag_from = None;
        }
        if let Some(prev) = self.drag_from {
            self.scroll -= my - prev;
            self.drag_from = Some(my);
        }

        // Arrow keys
        self.scroll += get_axis(KeyCode::ArrowUp, KeyCode::ArrowDown) * 500.0 * get_frame_time();

        self.scroll = self.scroll.clamp(0.0, (self.content_h - self.h).max(0.0));
    }

    /// Everything drawn after this (until `end`) goes into the frame.
    fn begin(&mut self, bg: Color) {
        self.cam.position = Vector2::new(0.0, self.scroll);
        set_camera(&self.cam);
        draw_rectangle(0.0, self.scroll, self.w, self.h, bg);
    }

    fn end(&self) {
        set_default_camera();

        let tex = self.cam.render_target.as_ref().unwrap().texture;
        draw_texture(tex, self.x, self.y, self.w, self.h, WHITE);
        draw_rectangle_lines(self.x, self.y, self.w, self.h, 2.0, WHITE);

        // Scrollbar
        let max_scroll = self.content_h - self.h;
        if max_scroll > 0.0 {
            let thumb_h = self.h * self.h / self.content_h;
            let thumb_y = self.y + self.scroll / max_scroll * (self.h - thumb_h);
            draw_rectangle_rounded(self.x + self.w - 8.0, thumb_y, 6.0, thumb_h, 3.0, GRAY);
        }
    }
}

#[flake]
async fn main() {
    const ROWS: usize = 30;
    const ROW_H: f32 = 50.0;

    let mut frame = ScrollFrame::new(50.0, 50.0, 300.0, 400.0, ROWS as f32 * ROW_H);

    loop {
        clear_background(BLACK);
        frame.update();

        // Which row is under the mouse (content-space)
        let (_, ly) = frame.local_mouse();
        let hovered_row = if frame.hovered() { Some((ly / ROW_H) as usize) } else { None };

        frame.begin(DARK_GRAY);
        for i in 0..ROWS {
            let color = if Some(i) == hovered_row { YELLOW } else if i % 2 == 0 { BLUE } else { LIGHT_BLUE };
            draw_rectangle_rounded(10.0, i as f32 * ROW_H + 5.0, 270.0, ROW_H - 10.0, 8.0, color);
        }
        frame.end();

        end_frame!();
    }
}