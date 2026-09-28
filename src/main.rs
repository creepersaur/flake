use flake::prelude::*;

fn main() -> FlakeResult {
    let font1 = load_ttf_font("src/Jetbrains.ttf");
    let font2 = load_ttf_font("src/vcr_mono.ttf");

    flake::run(|| {
        draw_text("Jetbrains Mono", 0.0, 0.0, 32.0, &font1, WHITE);
        draw_text("Supersonic Rocketship", 0.0, 32.0, 32.0, &font2, RED);
    })
}