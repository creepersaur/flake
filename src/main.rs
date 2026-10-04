use flake::prelude::*;

#[flake]
async fn main() {
    set_window_decorations(false);
    set_window_size(280, 36);
    set_window_level(WindowLevel::AlwaysOnTop);
    set_window_passthrough(true);
    set_window_enabled_buttons(WindowButtons::CLOSE);

    let mut pos = mouse_screen_position_vec2();

    loop {
        clear_background(TRANSPARENT);

        pos = pos.lerp(mouse_screen_position_vec2(), 0.02);
        set_window_position(pos.x as i32, pos.y as i32);

        draw_rich_text(vec2(20.0, 0.0), &[
            span("mouse x: ", ORANGE),
            span(&mouse_screen_position().0.to_string(), WHITE),
            span("\nmouse y: ", ORANGE),
            span(&mouse_screen_position().1.to_string(), WHITE),
        ]);


        next_frame().await;
    }
}

fn split_keep<'a>(s: &'a str, delims: &'a [char]) -> impl Iterator<Item = &'a str> {
    let mut rest = s;
    std::iter::from_fn(move || {
        let first = rest.chars().next()?;
        let end = if delims.contains(&first) {
            first.len_utf8() // delimiter becomes its own item
        } else {
            rest.find(delims).unwrap_or(rest.len()) // text up to the next delimiter
        };
        let (item, tail) = rest.split_at(end);
        rest = tail;
        Some(item)
    })
}
