use flake::prelude::*;

#[flake]
async fn main() {
    set_window_decorations(false);
    set_window_level(WindowLevel::AlwaysOnTop);

    loop {
        clear_background(WHITE);

        println!("{}", mouse_wheel_global().y);

        end_frame!();
    }
}