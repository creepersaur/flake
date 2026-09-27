#![allow(unused)]

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::global::{PENDING_CONFIG, STATE, ctx};
pub use crate::misc::math::*;
pub use crate::misc::rect::Rect;
pub use crate::shapes::color::*;
pub use crate::FlakeResult;
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;
/// # Out-facing API

/// Exits the running application, must be called while running.
///
/// # Examples
///
/// ```
/// flake_quit();
/// ```

pub fn flake_quit() {
    ctx(|s| s.exit())
}

/// # Window

/// Sets the title of the window.
///
/// # Examples
///
/// ```
/// set_window_title("Hello");
/// ```
pub fn set_window_title(title: &str) {
    if STATE.get().is_some() {
        return ctx(|s| s.set_window_title(title));
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.window_title = Some(title.into()));
}

/// Sets whether the window is visible or not.
///
/// # Examples
///
/// ```
/// set_window_visible(false); // hides the window
/// set_window_visible(true); // shows the window
/// ```
pub fn set_window_visible(visible: bool) {
    if STATE.get().is_some() {
        return ctx(|s| s.set_window_visible(visible));
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.window_visible = Some(visible));
}

/// Sets the icon of the window.
///
/// # Examples
///
/// ```
/// set_window_icon("path_to_icon");
/// ```
pub fn set_window_icon(path: &str) {
    let img = image::open(path)
        .expect("image icon does not exist at path")
        .into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();

    if STATE.get().is_some() {
        return ctx(|s| s.set_window_icon(rgba, width, height));
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.window_icon = Some((rgba, width, height)));
}

pub fn set_window_icon_bytes(bytes: &[u8]) {
    let img = image::load_from_memory(bytes)
        .expect("image icon does not exist at path")
        .into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();

    if STATE.get().is_some() {
        return ctx(|s| s.set_window_icon(rgba, width, height));
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.window_icon = Some((rgba, width, height)));
}

/// Sets position X of the window.
///
/// # Examples
///
/// ```
/// set_window_x(500);
/// ```
pub fn set_window_x(x: i32) {
    if STATE.get().is_some() {
        return ctx(|s| s.set_window_x(x));
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.window_x = Some(x));
}

/// Sets position Y of the window.
///
/// # Examples
///
/// ```
/// set_window_y(500);
/// ```
pub fn set_window_y(y: i32) {
    if STATE.get().is_some() {
        return ctx(|s| s.set_window_y(y));
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.window_y = Some(y));
}

/// Sets position of the window.
///
/// # Examples
///
/// ```
/// set_window_position(500, 500);
/// ```
pub fn set_window_position(x: i32, y: i32) {
    set_window_x(x);
    set_window_y(y);
}

/// Sets width of the window.
///
/// # Examples
///
/// ```
/// set_window_width(500);
/// ```
pub fn set_window_width(w: u32) {
    if STATE.get().is_some() {
        return ctx(|s| s.set_window_w(w));
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.window_width = Some(w));
}

/// Sets height of the window.
///
/// # Examples
///
/// ```
/// set_window_height(500);
/// ```
pub fn set_window_height(h: u32) {
    if STATE.get().is_some() {
        return ctx(|s| s.set_window_h(h));
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.window_height = Some(h));
}

/// Sets size of the window.
///
/// # Examples
///
/// ```
/// set_window_size(500, 500);
/// ```
pub fn set_window_size(w: u32, h: u32) {
    set_window_width(w);
    set_window_height(h);
}

/// # TIME

/// Gets the frames per second of the application.
/// (Limited by target framerate & fps cap). Uncap the FPS using `set_fps_capped(false)`.
///
/// # Examples
///
/// ```
/// println!("{}", get_fps());
/// ```
pub fn get_fps() -> f32 {
    ctx(|s| s.get_fps())
}

/// Sets target FPS of the application (Uses screen refresh rate by default).
///
/// # Examples
///
/// ```
/// set_target_fps(60);
/// ```
pub fn set_target_fps(fps: usize) {
    if STATE.get().is_some() {
        return ctx(|s| s.set_target_fps(fps));
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.target_fps = Some(fps));
}

/// Caps/unlocks the FPS of the application. (Capped by default)
///
/// # Examples
///
/// ```
/// set_fps_capped(false); // uncaps the FPS
/// set_fps_capped(true); // caps the FPS to target FPS
/// ```
pub fn set_fps_capped(capped: bool) {
    if STATE.get().is_some() {
        return ctx(|s| s.set_fps_capped(capped));
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.fps_capped = Some(capped));
}

/// Gets the time taken since last frame in seconds.
///
/// # Examples
///
/// ```
/// println!("{}", get_frame_time());
/// ```

pub fn get_frame_time() -> f32 {
    ctx(|s| s.get_frame_time())
}

/// Gets the amount of frames that have gone by since the start of the application.
///
/// # Examples
///
/// ```
/// println!("{}", get_frames());
/// ```

pub fn get_frames() -> usize {
    ctx(|s| s.get_frames())
}

/// ## INPUT

/// Returns the mouse x and y position in a tuple.
///
/// # Examples
///
/// ```
/// let (mx, my) = mouse_position();
/// ```

pub fn mouse_position() -> (f32, f32) {
    ctx(|s| s.mouse_state.get_position().into())
}

/// Returns the mouse x and y position in a `Vector2`.
///
/// # Examples
///
/// ```
/// let pos = mouse_position_vec2();
/// println!("{}, {}", pos.x, pos.y);
/// ```

pub fn mouse_position_vec2() -> Vector2 {
    ctx(|s| s.mouse_state.get_position())
}

/// Checks if a `MouseButton` is being held.
///
/// # Examples
///
/// ```
/// if is_mouse_button_down(MouseButton::Left) {
///     println!("Left mouse button is being held");
/// }
/// ```

pub fn is_mouse_button_down(btn: MouseButton) -> bool {
    ctx(|s| s.mouse_state.is_button_down(btn))
}

/// Checks if a `MouseButton` was just clicked this frame.
///
/// # Examples
///
/// ```
/// if is_mouse_button_clicked(MouseButton::Left) {
///     println!("User clicked left mouse button once");
/// }
/// ```

pub fn is_mouse_button_clicked(btn: MouseButton) -> bool {
    ctx(|s| s.mouse_state.is_button_clicked(btn))
}

/// Checks if a `MouseButton` was just released this frame.
///
/// # Examples
///
/// ```
/// if is_mouse_button_released(MouseButton::Left) {
///     println!("User let go of left mouse button once");
/// }
/// ```

pub fn is_mouse_button_released(btn: MouseButton) -> bool {
    ctx(|s| s.mouse_state.is_button_released(btn))
}

/// Checks if a key on the keyboard is being held.
///
/// # Examples
///
/// ```
/// println!("is Space being held: {}", is_key_down(KeyCode::Space));
/// ```

pub fn is_key_down(key: KeyCode) -> bool {
    ctx(|s| s.keyboard_state.is_key_down(key))
}

/// Checks if a key on the keyboard was just clicked.
///
/// # Examples
///
/// ```
/// if is_key_clicked(KeyCode::Space) {
///     println!("User clicked Space once");
/// }
/// ```

pub fn is_key_clicked(key: KeyCode) -> bool {
    ctx(|s| s.keyboard_state.is_key_clicked(key))
}
/// Checks if a key on the keyboard was just released.
///
/// # Examples
///
/// ```
/// if is_key_released(KeyCode::Space) {
/// println!("User let go of Space once"); /// }
/// ```

pub fn is_key_released(key: KeyCode) -> bool {
    ctx(|s| s.keyboard_state.is_key_released(key))
}

pub fn get_axis(left: KeyCode, right: KeyCode) -> f32 {
    ctx(|s| match (is_key_down(left), is_key_down(right)) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    })
}

/// ## DRAWING

/// Clears the background using a color.
///
/// # Examples
///
/// ```
/// clear_background(RED);
/// ```

pub fn clear_background(c: Color) {
    ctx(|s| {
        s.clear_color = c;
        s.draw_state.clear();
    });
}

/// Draws a filled rectangle with its top-left corner at `(x, y)`.
///
/// # Examples
///
/// ```
/// draw_rectangle(100.0, 100.0, 50.0, 80.0, RED);
/// ```

pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, c: Color) {
    ctx(|s| s.draw_state.draw_rectangle(x, y, w, h, c));
}

/// Draws a filled rectangle using a `Rect`.
///
/// # Examples
///
/// ```
/// draw_rectangle_from_rect(100.0, 100.0, 50.0, 80.0, RED);
/// ```
pub fn draw_rectangle_from_rect(rect: Rect, c: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, c);
}

/// Draws the outline of a rectangle with its top-left corner at `(x, y)`.
///
/// # Examples
///
/// ```
/// draw_rectangle_lines(100.0, 100.0, 50.0, 80.0, 4.0, WHITE);
/// ```

pub fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32, thickness: f32, c: Color) {
    ctx(|s| s.draw_state.draw_rectangle_lines(x, y, w, h, thickness, c));
}

/// Draws the outline of a rectangle using a `Rect`.
///
/// # Examples
///
/// ```
/// draw_rectangle_lines_from_rect(100.0, 100.0, 50.0, 80.0, 4.0, WHITE);
/// ```
pub fn draw_rectangle_lines_from_rect(rect: Rect, thickness: f32, c: Color) {
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, thickness, c);
}
/// Draws a filled rectangle rotated by `rotation` radians around `(x, y)`.
///
/// # Examples
///
/// ```
/// draw_rectangle_rotated(200.0, 200.0, 50.0, 80.0, std::f32::consts::FRAC_PI_4, GREEN);
/// ```

pub fn draw_rectangle_rotated(x: f32, y: f32, w: f32, h: f32, rotation: f32, c: Color) {
    ctx(|s| s.draw_state.draw_rectangle_rotated(x, y, w, h, rotation, c));
}

/// Draws the outline of a rectangle rotated by `rotation` radians around `(x, y)`.
///
/// # Examples
///
/// ```
/// draw_rectangle_lines_rotated(200.0, 200.0, 50.0, 80.0, 0.5, 3.0, WHITE);
/// ```

pub fn draw_rectangle_lines_rotated(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    thickness: f32,
    c: Color,
) {
    ctx(|s| {
        s.draw_state
            .draw_rectangle_lines_rotated(x, y, w, h, rotation, thickness, c)
    });
}

/// Draws a filled circle centered at `(x, y)` with radius `r`.
///
/// # Examples
///
/// ```
/// draw_circle(300.0, 300.0, 40.0, BLUE);
/// ```

pub fn draw_circle(x: f32, y: f32, r: f32, c: Color) {
    ctx(|s| s.draw_state.draw_circle(x, y, r, c));
}

/// Draws the outline of a circle centered at `(x, y)` with radius `r`.
///
/// # Examples
///
/// ```
/// draw_circle_lines(300.0, 300.0, 40.0, 3.0, WHITE);
/// ```

pub fn draw_circle_lines(x: f32, y: f32, r: f32, thickness: f32, c: Color) {
    ctx(|s| s.draw_state.draw_circle_lines(x, y, r, thickness, c));
}

/// Draws a filled triangle from three points.
///
/// # Examples
///
/// ```
/// draw_triangle(100.0, 200.0, 200.0, 200.0, 150.0, 100.0, YELLOW);
/// ```

pub fn draw_triangle(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, color: Color) {
    ctx(|s| s.draw_state.draw_triangle(x1, y1, x2, y2, x3, y3, color));
}

/// Draws the outline of a triangle from three points.
///
/// # Examples
///
/// ```
/// draw_triangle_lines(100.0, 200.0, 200.0, 200.0, 150.0, 100.0, 3.0, WHITE);
/// ```

pub fn draw_triangle_lines(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
    thickness: f32,
    color: Color,
) {
    ctx(|s| {
        s.draw_state
            .draw_triangle_lines(x1, y1, x2, y2, x3, y3, thickness, color)
    });
}

/// Draws a line segment from `(x1, y1)` to `(x2, y2)`.
///
/// # Examples
///
/// ```
/// draw_line(0.0, 0.0, 200.0, 150.0, 4.0, WHITE);
/// ```

pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
    ctx(|s| s.draw_state.draw_line(x1, y1, x2, y2, thickness, color))
}

/// Draws connected line segments through `points` with mitered joins.
/// If `closed` is true, the last point connects back to the first.
///
/// # Examples
///
/// ```
/// let pts = [Vector2::new(50.0, 50.0), Vector2::new(150.0, 80.0), Vector2::new(100.0, 160.0)];
/// draw_poly_line(&pts, 3.0, WHITE, true);
/// ```

pub fn draw_poly_line(points: &[Vector2], thickness: f32, color: Color, closed: bool) {
    ctx(|s| {
        s.draw_state
            .draw_poly_line_miter(points, thickness, color, closed)
    })
}

/// Draws connected line segments through `points` with rounded joints and caps.
/// If `closed` is true, the last point connects back to the first.
///
/// # Examples
///
/// ```
/// let pts = [Vector2::new(50.0, 50.0), Vector2::new(150.0, 80.0), Vector2::new(100.0, 160.0)];
/// draw_poly_line_rounded(&pts, 6.0, WHITE, false);
/// ```

pub fn draw_poly_line_rounded(points: &[Vector2], thickness: f32, color: Color, closed: bool) {
    ctx(|s| {
        s.draw_state
            .draw_poly_line(points, thickness, color, closed, true)
    })
}

/// Draws a filled convex polygon from `points` as a triangle fan.
/// Fewer than 3 points draws nothing. Use `draw_polygon_concave` for concave shapes.
///
/// # Examples
///
/// ```
/// let pts = [Vector2::new(100.0, 100.0), Vector2::new(200.0, 100.0),
/// Vector2::new(220.0, 180.0), Vector2::new(80.0, 180.0)];/// draw_polygon(&pts, GREEN);
/// ```

pub fn draw_polygon(points: &[Vector2], thickness: f32, color: Color) {
    ctx(|s| s.draw_state.draw_polygon(points, color))
}

/// Draws a filled polygon that may be concave, using ear clipping.
/// Self-intersecting polygons may not render correctly.
///
/// # Examples
///
/// ```
/// let pts = [Vector2::new(100.0, 100.0), Vector2::new(200.0, 100.0),
/// Vector2::new(150.0, 150.0), Vector2::new(200.0, 200.0), /// Vector2::new(100.0, 200.0)];
/// draw_polygon_concave(&pts, RED);
/// ```

pub fn draw_polygon_concave(points: &[Vector2], thickness: f32, color: Color) {
    ctx(|s| s.draw_state.draw_polygon_concave(points, color))
}

/// Draws text with its top-left corner at `(x, y)` and the given font `size`.
///
/// # Examples
///
/// ```
/// draw_text("Hello", 10.0, 10.0, 32.0, WHITE);
/// ```

pub fn draw_text(t: &str, x: f32, y: f32, size: f32, c: Color) {
    ctx(|s| s.draw_state.draw_text(t, x, y, size, c));
}

pub fn draw_arrow(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, head_size: f32, c: Color) {
    ctx(|s| {
        s.draw_state
            .draw_arrow(x1, y1, x2, y2, thickness, head_size, c)
    });
}
