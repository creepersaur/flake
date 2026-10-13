use flake::prelude::*;

#[flake(skip_taskbar = true)]
async fn main() {
    let screensize = screen_size();
    set_window_size(screensize.x as u32 - 1, screensize.x as u32 - 1);
    set_window_position(0, 0);
    set_window_decorations(false);
    set_window_passthrough(true);
    set_window_level(WindowLevel::AlwaysOnTop);

    loop {
        clear_background(TRANSPARENT);

        draw_rich_text(mouse_position_vec2() + Vector2::new(20.0, 20.0), &[
            span("Mouse x: ", ORANGE),
            span(&format!("{}", mouse_screen_position().0), WHITE),
            span("\nMouse y: ", ORANGE),
            span(&format!("{}", mouse_screen_position().1), WHITE),
        ]);

        next_frame().await
    }
}
