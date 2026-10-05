use flake::prelude::*;

#[flake]
async fn main() {
    let mut img = Image::new(2, 2, FilterType::Nearest);
    img.set_pixel(0, 0, RED);
    img.set_pixel(1, 0, YELLOW);
    img.set_pixel(0, 1, GREEN);
    img.set_pixel(1, 1, BLUE);

    let tex1 = load_texture_from_image(img);

    loop {
        clear_background(BLACK);
        draw_texture(tex1, 100.0, 100.0, 100.0, 100.0, WHITE);

        next_frame().await;
    }
}
