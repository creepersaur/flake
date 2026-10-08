#![allow(unused)]

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::draw_state::shapes::Shape;
pub use crate::draw_state::shapes::color::*;
use crate::global::{PENDING_CONFIG, STATE, ctx};
pub use crate::misc::flake_image::Image;
use crate::misc::font::Font;
pub use crate::misc::math::*;
pub use crate::misc::rect::Rect;
use crate::misc::span::RichtextSection;
pub use crate::misc::span::Span;
pub use crate::misc::timer::Timer;
pub use crate::misc::*;
use crate::model::instance::Instance;
pub use crate::model::texture::DrawTextureParams;
pub use crate::model::texture::FilterType;
use crate::model::texture::{GpuTexture, Texture};
pub use crate::runtime::next_frame::next_frame;
pub use crate::{FlakeResult, flake};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb, Rgba};
use std::borrow::Borrow;
use std::sync::Arc;
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;
use winit::window::Window;
pub use winit::window::{WindowButtons, WindowLevel};

// -- Out-facing API --------------------------------------------------------------------------------------------------------------

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

/// Sleeps (waits) for a set amount of seconds.
/// (Will pause the window thread as well)
/// App may become "not responding" if time is too high.
/// Good for games like tetris/snake.
///
/// # Examples
///
/// ```
/// flake_sleep(0.1);
/// ```
pub fn flake_sleep(seconds: f32) {
    std::thread::sleep(std::time::Duration::from_secs_f32(seconds));
}

/// Creates a `Timer` whose time goes down each frame. Can be paused. Check `Timer` docs for more info.
///
/// # Examples
///
/// ```
/// let mut my_timer = add_timer(Timer::new(5.0));
///
/// my_timer.on_completed(|| {
///     println!("Timer got completed");
/// });
/// ```
pub fn add_timer(timer: Timer) -> Timer {
    if STATE.get().is_some() {
        return ctx(|s| {
            if !s.timers.contains(&timer) {
                s.add_timer(timer.clone());
            }
            timer
        });
    }

    PENDING_CONFIG.with_borrow_mut(|s| {
        if !s.timers.contains(&timer) {
            s.timers.push(timer.clone())
        }
    });

    timer
}

/// Gets the winit window used by the application
///
/// # Examples
///
/// ```
/// let win = get_window();
/// win.set_transparent(true);
/// ```
pub fn get_window() -> Arc<Window> {
    ctx(|s| s.get_window())
}

/// Sets whether window catches mouse events.
pub fn set_window_passthrough(passthrough: bool) {
    if STATE.get().is_some() {
        return ctx(|s| s.set_window_passthrough(passthrough));
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.window_passthrough = Some(passthrough));
}

/// Sets whether the window is AlwaysOnTop, AlwaysOnBottom, or Normal
pub fn set_window_level(level: WindowLevel) {
    if STATE.get().is_some() {
        return ctx(|s| s.set_window_level(level));
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.window_level = Some(level));
}

/// Sets which window buttons are enabled. Any button not included is disabled
/// (e.g. `WindowButtons::CLOSE` leaves only the close button).
pub fn set_window_enabled_buttons(window_buttons: WindowButtons) {
    ctx(|s| s.get_window().set_enabled_buttons(window_buttons));
}

/// Sets whether the window is transparent.
/// **Run this before `flake::run()`** as it might not work after the window has been created.
pub fn set_window_transparent(transparent: bool) {
    if STATE.get().is_some() {
        return get_window().set_transparent(transparent);
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.window_transparent = Some(transparent));
}

/// Sets whether the window decorations (titlebar, border, etc.) are enabled
pub fn set_window_decorations(enabled: bool) {
    if STATE.get().is_some() {
        return get_window().set_decorations(enabled);
    }

    PENDING_CONFIG.with_borrow_mut(|s| s.window_decorations = Some(enabled));
}

// -- Window --------------------------------------------------------------------------------------------------------------

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

/// Sets the window icon from encoded image bytes (PNG, etc.). Panics if the bytes aren't a valid image.
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

/// Gets width of the window
pub fn window_width() -> f32 {
    ctx(|s| s.window_width())
}

/// Gets height of the window
pub fn window_height() -> f32 {
    ctx(|s| s.window_height())
}

/// Gets size of the window
pub fn window_size() -> Vector2 {
    ctx(|s| Vector2::new(s.window_width(), s.window_height()))
}

/// Gets position of the window
pub fn window_position() -> (f32, f32) {
    ctx(|s| s.window_position())
}

/// Gets position of the window as Vector2<f32>
pub fn window_position_vec2() -> Vector2 {
    ctx(|s| s.window_position().into())
}

/// Gets X position of the window
pub fn window_x() -> f32 {
    ctx(|s| s.window_x())
}

/// Gets Y position of the window
pub fn window_y() -> f32 {
    ctx(|s| s.window_y())
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
    if STATE.get().is_some() {
        return ctx(|s| s.set_window_size(w, h));
    }

    PENDING_CONFIG.with_borrow_mut(|s| {
        s.window_width = Some(w);
        s.window_height = Some(h);
    });
}

/// Sets whether the window can be resized using the resize handles.
pub fn set_window_resizable(resizeable: bool) {
    ctx(|s| s.window.set_resizable(resizeable));
}

// -- TIME --------------------------------------------------------------------------------------------------------------

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

// -- INPUT --------------------------------------------------------------------------------------------------------------

/// Returns the window-relative mouse x and y position in a tuple.
///
/// # Examples
///
/// ```
/// let (mx, my) = mouse_position();
/// ```
pub fn mouse_position() -> (f32, f32) {
    ctx(|s| s.mouse_state.get_position().into())
}

/// Returns the window-relative mouse x and y position in a `Vector2`.
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

/// Returns the mouse x and y position on the screen instead of window-relative as Vector2.
pub fn mouse_screen_position_vec2() -> Vector2 {
    ctx(|s| s.mouse_state.get_screen_position())
}

/// Returns the mouse x and y position on the screen instead of window-relative.
pub fn mouse_screen_position() -> (f32, f32) {
    ctx(|s| s.mouse_state.get_screen_position().into())
}

/// Requests keyboard focus for the window.
pub fn focus_window() {
    get_window().focus_window();
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
/// if is_mouse_button_pressed(MouseButton::Left) {
///     println!("User clicked left mouse button once");
/// }
/// ```
pub fn is_mouse_button_pressed(btn: MouseButton) -> bool {
    ctx(|s| s.mouse_state.is_button_pressed(btn))
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
/// if is_key_pressed(KeyCode::Space) {
///     println!("User clicked Space once");
/// }
/// ```
pub fn is_key_pressed(key: KeyCode) -> bool {
    ctx(|s| s.keyboard_state.is_key_pressed(key))
}

/// Checks if a key on the keyboard was just released.
///
/// # Examples
///
/// ```
/// if is_key_released(KeyCode::Space) {
///     println!("User let go of Space");
/// }
/// ```
pub fn is_key_released(key: KeyCode) -> bool {
    ctx(|s| s.keyboard_state.is_key_released(key))
}

/// Returns -1.0 if only `left` is held, 1.0 if only `right` is held, otherwise 0.0.
pub fn get_axis(left: KeyCode, right: KeyCode) -> f32 {
    ctx(|s| match (is_key_down(left), is_key_down(right)) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    })
}

/// Returns a direction from four keys. Up is -1.0 on Y (screen coordinates).
/// The result is not normalized, so diagonals have length √2.
pub fn get_vector(left: KeyCode, right: KeyCode, up: KeyCode, down: KeyCode) -> Vector2 {
    ctx(|s| {
        Vector2::new(
            match (is_key_down(left), is_key_down(right)) {
                (true, false) => -1.0,
                (false, true) => 1.0,
                _ => 0.0,
            },
            match (is_key_down(up), is_key_down(down)) {
                (true, false) => -1.0,
                (false, true) => 1.0,
                _ => 0.0,
            },
        )
    })
}

// -- DRAWING --------------------------------------------------------------------------------------------------------------

/// Clears the background using a color.
///
/// # Examples
///
/// ```
/// clear_background(RED);
/// ```

pub fn clear_background(color: Color) {
    if STATE.get().is_some() {
        return ctx(|s| {
            s.clear_color = Some(color);
            s.draw_state.clear();
        });
    }

    PENDING_CONFIG.with_borrow_mut(|c| c.window_clear_color = Some(color));
}

/// Draws a filled rectangle with its top-left corner at `(x, y)`.
///
/// # Examples
///
/// ```
/// draw_rectangle(100.0, 100.0, 50.0, 80.0, RED);
/// ```

pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: Color) {
    draw_rectangle_rounded(x, y, w, h, 0.0, color);
}

/// Draws a **rounded**, filled rectangle with its top-left corner at `(x, y)`.
/// All corners have the same radius. Use `draw_rectangle_rounded_ex()` to set all the radii separately.
///
/// # Examples
///
/// ```
/// draw_rectangle_rounded(100.0, 100.0, 50.0, 80.0, 20.0, RED); // 20.0 is the radius of each corner
/// ```
pub fn draw_rectangle_rounded(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    draw_rectangle_rounded_ex(x, y, w, h, radius, radius, radius, radius, color)
}

/// Draws a **rounded**, filled rectangle with its top-left corner at `(x, y)`
/// with the ability to set each corner's radius manually.
///
/// # Examples
///
/// ```
/// draw_rectangle_rounded_ex(
///     x, y,
///     w, h,
///     top_left, top_right,
///     bottom_left, bottom_right,
///     RED
/// );
/// ```
pub fn draw_rectangle_rounded_ex(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    top_left: f32,
    top_right: f32,
    bottom_left: f32,
    bottom_right: f32,
    color: Color,
) {
    ctx(|s| {
        s.draw_state.draw_rectangle(
            x,
            y,
            w,
            h,
            top_left,
            top_right,
            bottom_left,
            bottom_right,
            color,
        )
    });
}

/// Draws a filled rectangle using a `Rect`.
///
/// # Examples
///
/// ```
/// draw_rectangle_from_rect(rect, RED);
/// ```
pub fn draw_rectangle_from_rect(rect: Rect, color: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
}

/// Draws a **rounded**, filled rectangle using a `Rect` where all corners have the same radius.
/// Use `draw_rectangle_rounded_from_rect_ex()` to set all radii separately.
///
/// # Examples
///
/// ```
/// draw_rectangle_rounded_from_rect(rect, 20.0, RED); // 20.0 is the radius
/// ```
pub fn draw_rectangle_rounded_from_rect(rect: Rect, radius: f32, color: Color) {
    draw_rectangle_rounded(rect.x, rect.y, rect.w, rect.h, radius, color);
}

/// Draws **rounded**, outlines of a rectangle using a `Rect` where all corners have the same radius.
/// Use `draw_rectangle_lines_rounded_from_rect_ex()` to set all radii separately.
///
/// # Examples
///
/// ```
/// draw_rectangle_lines_rounded_from_rect(rect, 20.0, RED); // 20.0 is the radius
/// ```
pub fn draw_rectangle_lines_rounded_from_rect(
    rect: Rect,
    radius: f32,
    thickness: f32,
    color: Color,
) {
    draw_rectangle_lines_rounded(rect.x, rect.y, rect.w, rect.h, radius, thickness, color);
}

/// Draws **rounded**, outlines of a rectangle using a `Rect` where all corners have the same radius.
/// Use `draw_rectangle_lines_rounded_from_rect_ex()` to set all radii separately.
///
/// # Examples
///
/// ```
/// draw_rectangle_lines_rounded_from_rect(rect, 20.0, RED); // 20.0 is the radius
/// ```
pub fn draw_rectangle_lines_rounded_from_rect_ex(
    rect: Rect,
    top_left: f32,
    top_right: f32,
    bottom_left: f32,
    bottom_right: f32,
    thickness: f32,
    color: Color,
) {
    draw_rectangle_lines_rounded_ex(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        top_left,
        top_right,
        bottom_left,
        bottom_right,
        thickness,
        color,
    );
}

/// Draws a **rounded**, filled rectangle using a `Rect`,
/// with the ability to set each corner's radius separately.
///
/// # Examples
///
/// ```
/// draw_rectangle_rounded_from_rect_ex(
///     rect,
///     top_left, top_right,
///     bottom_left, bottom_right,
///     RED
/// );
/// ```
pub fn draw_rectangle_rounded_from_rect_ex(
    rect: Rect,
    top_left: f32,
    top_right: f32,
    bottom_left: f32,
    bottom_right: f32,
    color: Color,
) {
    draw_rectangle_rounded_ex(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        top_left,
        top_right,
        bottom_left,
        bottom_right,
        color,
    );
}

/// Draws the outline of a rectangle with its top-left corner at `(x, y)`.
///
/// # Examples
///
/// ```
/// draw_rectangle_lines(100.0, 100.0, 50.0, 80.0, 4.0, WHITE);
/// ```
pub fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
    draw_rectangle_lines_rounded(x, y, w, h, thickness, 0.0, color)
}

/// Draws the outline of a rectangle using a `Rect`.
///
/// # Examples
///
/// ```
/// draw_rectangle_lines_from_rect(rect, 4.0, WHITE);
/// ```
pub fn draw_rectangle_lines_from_rect(rect: Rect, thickness: f32, color: Color) {
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, thickness, color);
}

/// Draws the outline of a **rounded** rectangle with its top-left corner at `(x, y)`.
/// All corners have the same radius. The stroke is centered on the rectangle's edge.
/// Use `draw_rectangle_lines_rounded_ex()` to set each corner's radius separately.
///
/// # Examples
///
/// ```
/// draw_rectangle_lines_rounded(100.0, 100.0, 50.0, 80.0, 12.0, 3.0, WHITE);
/// // radius = 12.0, thickness = 3.0
/// ```
pub fn draw_rectangle_lines_rounded(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    thickness: f32,
    color: Color,
) {
    draw_rectangle_lines_rounded_ex(x, y, w, h, radius, radius, radius, radius, thickness, color);
}

/// Draws the outline of a **rounded** rectangle with its top-left corner at `(x, y)`,
/// with the ability to set each corner's radius separately.
/// Radii are clamped to half the shorter side.
///
/// # Examples
///
/// ```
/// draw_rectangle_lines_rounded_ex(
///     100.0, 100.0, 50.0, 80.0,
///     20.0, 0.0,   // top_left, top_right
///     0.0, 20.0,   // bottom_left, bottom_right
///     3.0, WHITE,  // thickness, color
/// );
/// ```
pub fn draw_rectangle_lines_rounded_ex(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    top_left: f32,
    top_right: f32,
    bottom_left: f32,
    bottom_right: f32,
    thickness: f32,
    color: Color,
) {
    ctx(|s| {
        s.draw_state.draw_rectangle_lines(
            x,
            y,
            w,
            h,
            top_left,
            top_right,
            bottom_left,
            bottom_right,
            thickness,
            color,
        )
    });
}

/// Draws a filled rectangle of size `w` x `h` centered at `(x, y)`,
/// rotated by `rotation` radians around that center.
///
/// # Examples
///
/// ```
/// draw_rectangle_rotated(200.0, 200.0, 50.0, 80.0, std::f32::consts::FRAC_PI_4, GREEN);
/// ```
pub fn draw_rectangle_rotated(x: f32, y: f32, w: f32, h: f32, rotation: f32, color: Color) {
    ctx(|s| {
        s.draw_state
            .draw_rectangle_rotated(x, y, w, h, rotation, 0.0, 0.0, 0.0, 0.0, color)
    });
}

/// Draws the outline of a rectangle with its top-left corner at `(x, y)` (before rotation),
/// rotated by `rotation` radians around its center.
pub fn draw_rectangle_lines_rotated(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    thickness: f32,
    color: Color,
) {
    ctx(|s| {
        s.draw_state.draw_rectangle_lines_rotated(
            x, y, w, h, rotation, 0.0, 0.0, 0.0, 0.0, thickness, color,
        )
    });
}

/// Draws a rounded, filled rectangle centered at `(x, y)`,
/// rotated by `rotation` radians around that center.
/// All corners have the same radius.
pub fn draw_rectangle_rounded_rotated(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    radius: f32,
    color: Color,
) {
    draw_rectangle_rounded_rotated_ex(x, y, w, h, rotation, radius, radius, radius, radius, color);
}

pub fn draw_rectangle_rounded_rotated_ex(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    top_left: f32,
    top_right: f32,
    bottom_left: f32,
    bottom_right: f32,
    color: Color,
) {
    ctx(|s| {
        s.draw_state.draw_rectangle_rotated(
            x,
            y,
            w,
            h,
            rotation,
            top_left,
            top_right,
            bottom_left,
            bottom_right,
            color,
        )
    });
}

/// Draws the outline of a rounded rectangle centered at `(x, y)`,
/// rotated by `rotation` radians around that center.
/// All corners have the same radius.
pub fn draw_rectangle_lines_rounded_rotated(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    radius: f32,
    thickness: f32,
    color: Color,
) {
    draw_rectangle_lines_rounded_rotated_ex(
        x, y, w, h, rotation, radius, radius, radius, radius, thickness, color,
    );
}

/// Draws the outline of a rounded rectangle centered at `(x, y)`,
/// rotated by `rotation` radians around that center.
/// Can set each corner's radius separately.
pub fn draw_rectangle_lines_rounded_rotated_ex(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    top_left: f32,
    top_right: f32,
    bottom_left: f32,
    bottom_right: f32,
    thickness: f32,
    color: Color,
) {
    ctx(|s| {
        s.draw_state.draw_rectangle_lines_rotated(
            x,
            y,
            w,
            h,
            rotation,
            top_left,
            top_right,
            bottom_left,
            bottom_right,
            thickness,
            color,
        )
    });
}

/// Draws a filled circle centered at `(x, y)` with radius `r`.
///
/// # Examples
///
/// ```
/// draw_circle(300.0, 300.0, 40.0, BLUE);
/// ```
pub fn draw_circle(x: f32, y: f32, r: f32, color: Color) {
    ctx(|s| s.draw_state.draw_circle(x, y, r, color));
}

/// Draws the outline of a circle centered at `(x, y)` with radius `r`.
///
/// # Examples
///
/// ```
/// draw_circle_lines(300.0, 300.0, 40.0, 3.0, WHITE);
/// ```

pub fn draw_circle_lines(x: f32, y: f32, r: f32, thickness: f32, color: Color) {
    ctx(|s| s.draw_state.draw_circle_lines(x, y, r, thickness, color));
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
/// let pts = [
///     Vector2::new(100.0, 100.0), Vector2::new(200.0, 100.0),
///     Vector2::new(220.0, 180.0), Vector2::new(80.0, 180.0),
/// ];
/// draw_polygon(&pts, GREEN);
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
/// let pts = [
///     Vector2::new(100.0, 100.0), Vector2::new(200.0, 100.0),
///     Vector2::new(150.0, 150.0), Vector2::new(200.0, 200.0),
///     Vector2::new(100.0, 200.0),
/// ];
/// draw_polygon_concave(&pts, RED);
/// ```
pub fn draw_polygon_concave(points: &[Vector2], thickness: f32, color: Color) {
    ctx(|s| s.draw_state.draw_polygon_concave(points, color))
}

/// Draws text with its top-left corner at `(x, y)` with the given font, size, and color.
///
/// # Examples
///
/// ```
/// draw_text("Hello", 10.0, 10.0, 32.0, None, WHITE);           // default font
/// draw_text("Hello", 10.0, 10.0, 32.0, Some(&font), WHITE);    // loaded font
/// ```
pub fn draw_text(t: &str, x: f32, y: f32, size: f32, font: Option<&Font>, color: Color) {
    ctx(|s| s.draw_state.draw_text(&s.fonts, t, x, y, size, font, color));
}

/// Draws a collection of `Span`s in sequence where each Span can have different properties such as
/// - text
/// - color
/// - size
/// - font
/// The `spans` should implement IntoIterator. Which means you can just pass an iterator.
///
/// ## Examples
///
/// ```
/// draw_rich_text(vec2(10.0, 10.0), &[
///     span("Hello there\n", WHITE),
///     span("My name is ", WHITE),
///     span("John Doe", RED)
///         .with_color(RED)
///         .with_size(32.0)
/// ]);
/// ```
pub fn draw_rich_text<'a, I>(position: Vector2, spans: I)
where
    I: IntoIterator,
    I::Item: Borrow<Span<'a>>,
{
    ctx(|s| s.draw_state.draw_richtext(position, spans))
}

/// Loads a `.ttf`/`.otf` font from path and returns the `Font` object.
/// (Do not load the font every frame, it will lag)
///
/// # Examples
///
/// ```
/// // Put this before the `flake::run()` to avoid loading it each frame
/// let font = load_font_from_path("src/Arial.ttf");
///
/// flake::run(|| {
///     draw_text("Hello", 10.0, 10.0, 32.0, Some(&font), WHITE)
/// })
/// ```
pub fn load_font_from_path(path: &str) -> Font {
    if STATE.get().is_some() {
        return ctx(|s| {
            s.load_font_from_path(path);
            s.fonts[s.fonts.len() - 1].clone()
        });
    }

    PENDING_CONFIG.with_borrow_mut(|s| {
        let font = Font::from_path(s.fonts.len() + 1, path);
        s.fonts.push(font.clone());
        font
    })
}

/// Loads a `.ttf`/`.otf` font from bytes and returns the `Font` object.
/// (Do not load the font every frame, it will lag)
///
/// # Examples
///
/// ```
/// // Put this before the `flake::run()` to avoid loading it each frame
/// let font = load_font_from_bytes(include_bytes!("Arial.ttf"));
///
/// flake::run(|| {
///     draw_text("Hello", 10.0, 10.0, 32.0, Some(&font), WHITE)
/// })
/// ```
pub fn load_font_from_bytes(bytes: &[u8]) -> Font {
    if STATE.get().is_some() {
        return ctx(|s| {
            s.load_font_from_bytes(bytes);
            s.fonts[s.fonts.len() - 1].clone()
        });
    }

    PENDING_CONFIG.with_borrow_mut(|s| {
        let font = Font::from_bytes(s.fonts.len() + 1, bytes);
        s.fonts.push(font.clone());
        font
    })
}

/// Draws a line from `(x1, y1)` to `(x2, y2)` with a triangular head at the end.
/// The tip extends `head_size` past `(x2, y2)`.
pub fn draw_arrow(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    thickness: f32,
    head_size: f32,
    color: Color,
) {
    ctx(|s| {
        s.draw_state
            .draw_arrow(x1, y1, x2, y2, thickness, head_size, color)
    });
}

/// Returns the size of the monitor the window is on, or the window size if it can't be determined.
pub fn screen_size() -> Vector2 {
    let win = get_window();
    match win.current_monitor() {
        Some(m) => {
            let s = m.size();
            Vector2::new(s.width as f32, s.height as f32)
        }
        None => window_size(),
    }
}

pub fn screen_width() -> f32 {
    screen_size().x
}
pub fn screen_height() -> f32 {
    screen_size().y
}

// -- TEXTURES --------------------------------------------------------------------------------------------------------------

/// Creates an empty texture of the given size. Fill it with `update_texture`.
pub fn create_empty_texture(width: u32, height: u32, filter_type: FilterType) -> Texture {
    ctx(|s| {
        let gpu = GpuTexture::empty(&s.device, width, height, filter_type)
            .expect("Couldn't create empty GpuTexture");

        Texture {
            id: s.add_gpu_texture(gpu),
            width,
            height,
        }
    })
}

/// Loads a texture from an image file using linear filtering. Panics if the file can't be read or decoded.
pub fn load_texture(path: &str) -> Texture {
    ctx(|s| {
        let image = image::load_from_memory(&std::fs::read(path).unwrap()).unwrap();
        let width = image.width();
        let height = image.height();

        Texture {
            id: s.add_texture(image, FilterType::Linear),
            width,
            height,
        }
    })
}

/// Same as `load_texture`, with an explicit `FilterType` (e.g. nearest for pixel art).
pub fn load_texture_with_filter_type(path: &str, filter_type: FilterType) -> Texture {
    ctx(|s| {
        let image = image::load_from_memory(&std::fs::read(path).unwrap()).unwrap();
        let width = image.width();
        let height = image.height();

        Texture {
            id: s.add_texture(image, filter_type),
            width,
            height,
        }
    })
}

/// Loads a texture from encoded image bytes (e.g. `include_bytes!`) using linear filtering.
pub fn load_texture_from_bytes(bytes: &[u8]) -> Texture {
    ctx(|s| {
        let image = image::load_from_memory(bytes).unwrap();
        let (width, height) = image.dimensions();

        Texture {
            id: s.add_texture(image, FilterType::Linear),
            width,
            height,
        }
    })
}

/// Loads a texture from raw RGBA `f32` pixels. `data.len()` must equal `width * height * 4`, otherwise this panics.
pub fn load_texture_from_bytes_with_filter_type(bytes: &[u8], filter_type: FilterType) -> Texture {
    ctx(|s| {
        let image = image::load_from_memory(bytes).unwrap();
        let (width, height) = image.dimensions();

        Texture {
            id: s.add_texture(image, filter_type),
            width,
            height,
        }
    })
}

/// Loads a texture from raw RGBA `f32` pixels. `data.len()` must equal `width * height * 4`, otherwise this panics.
pub fn load_texture_from_data(width: u32, height: u32, data: &[f32]) -> Texture {
    ctx(|s| {
        let img_buf = ImageBuffer::<Rgba<f32>, Vec<f32>>::from_raw(width, height, data.to_vec())
            .expect("Buffer size mismatch");

        let dynamic_img = DynamicImage::ImageRgba32F(img_buf);

        Texture {
            id: s.add_texture(dynamic_img, FilterType::Linear),
            width,
            height,
        }
    })
}

/// Loads a texture from raw RGBA `f32` pixels.
/// `data.len()` must equal `width * height * 4`, otherwise this panics.
/// (Use nearest for pixel art)
pub fn load_texture_from_data_with_filter_type(
    width: u32,
    height: u32,
    data: &[f32],
    filter_type: FilterType,
) -> Texture {
    ctx(|s| {
        let img_buf = ImageBuffer::<Rgba<f32>, Vec<f32>>::from_raw(width, height, data.to_vec())
            .expect("Buffer size mismatch");

        let dynamic_img = DynamicImage::ImageRgba32F(img_buf);

        Texture {
            id: s.add_texture(dynamic_img, filter_type),
            width,
            height,
        }
    })
}

/// Creates a texture from an `Image`.
pub fn load_texture_from_image(image: &Image) -> Texture {
    let tex = create_empty_texture(image.width, image.height, image.filter_type);
    update_texture(tex, image);

    tex
}

/// Draws a texture stretched to `w` x `h` with its top-left at `(x, y)`.
/// `tint` multiplies the texture color (use `WHITE` for none).
pub fn draw_texture(texture: Texture, x: f32, y: f32, w: f32, h: f32, tint: Color) {
    ctx(|s| s.draw_state.draw_texture(texture, x, y, w, h, tint))
}

/// Draws a texture with extra options from `DrawTextureParams`: destination size,
/// source rect, horizontal/vertical flip, rotation (radians), pivot, and tint.
pub fn draw_texture_ex(texture: Texture, x: f32, y: f32, p: DrawTextureParams) {
    ctx(|s| s.draw_state.draw_texture_ex(texture, x, y, p))
}

/// Replaces the pixels of an existing texture with those from `image`.
pub fn update_texture(texture: Texture, image: &Image) {
    ctx(|s| s.update_texture(texture.id, image))
}
