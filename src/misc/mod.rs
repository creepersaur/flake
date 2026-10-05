use crate::prelude::{Color, Quaternion, Rect, Span, Timer, Vector2, Vector3, Vector4};

pub mod math;
pub mod rect;
pub mod font;
pub mod timer;
pub mod span;
pub mod flake_image;

macro_rules! simplified_constructors {
    ($($name:ident $(<$lt:lifetime>)? ($($arg:ident: $ty:ty),*) -> $ret:ty => $new:path;)*) => {
        $(
            #[inline]
            pub fn $name $(<$lt>)? ($($arg: $ty),*) -> $ret {
                $new($($arg),*)
            }
        )*
    };
}

simplified_constructors! {
    vec2(x: f32, y: f32) -> Vector2 => Vector2::new;
    vec3(x: f32, y: f32, z: f32) -> Vector3 => Vector3::new;
    vec4(x: f32, y: f32, z: f32, w: f32) -> Vector4 => Vector4::new;
    quat(x: f32, y: f32, z: f32, w: f32) -> Quaternion => Quaternion::new;

    rect(x: f32, y: f32, w: f32, h: f32) -> Rect => Rect::new;

    timer(seconds: f32) -> Timer => Timer::new;
    timer_repeating(seconds: f32) -> Timer => Timer::new_repeating;

    span<'a>(text: &'a str, color: Color) -> Span<'a> => Span::new;
}