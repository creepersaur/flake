use flake::prelude::*;

#[flake]
async fn main() {
    set_window_decorations(false);
    set_window_level(WindowLevel::AlwaysOnTop);

    loop {
        clear_background(WHITE);

        let h = get_pressed_keys().collect::<Vec<_>>();
        if h.len() > 0 {
            println!("{}, {:?}", h.len(), last_key_name());
        }

        end_frame!();
    }
}