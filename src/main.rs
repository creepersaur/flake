use flake::prelude::*;

fn main() -> FlakeResult {
    set_window_title("Hello");
    set_window_size(300, 300);
    set_window_icon("src/cupcake.png");

    run!(RED => {
    })
}
