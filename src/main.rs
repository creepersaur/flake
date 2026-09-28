use flake::prelude::*;

fn main() -> FlakeResult {
    let font1 = load_ttf_font("src/Jetbrains.ttf");
    let font2 = load_ttf_font("src/br_cobane.otf");

    flake::run(|| {
        draw_text("Hello", 10.0, 10.0, 32.0, Some(&font2), WHITE);
        draw_text("           World", 10.0, 10.0, 32.0, Some(&font2), RED);

        draw_text("Testing out different fonts", 10.0, 40.0, 20.0, Some(&font1), GRAY);
    })
}