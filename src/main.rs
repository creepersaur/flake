use flake::prelude::*;

#[flake]
async fn main() {
    let mut img = Image::from_pixels(3, 7, FilterType::Nearest, vec![
        DARK_RED, RED, LIGHT_RED,
        DARK_ORANGE, ORANGE, LIGHT_ORANGE,
        DARK_YELLOW, YELLOW, LIGHT_YELLOW,
        DARK_GREEN, GREEN, LIGHT_GREEN,
        DARK_CYAN, CYAN, LIGHT_CYAN,
        DARK_BLUE, BLUE, LIGHT_BLUE,
        DARK_PURPLE, PURPLE, LIGHT_PURPLE,
    ]);
    let tex1 = load_texture_from_image(&img);
    
    img.set_pixel(0, 0, WHITE);
    tex1.update(&img);

    loop {
        clear_background(BLACK);
        draw_texture(tex1, 100.0, 100.0, 100.0, 100.0, WHITE);

        next_frame().await;
    }
}
