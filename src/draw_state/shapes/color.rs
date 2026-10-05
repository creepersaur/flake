use crate::prelude::{Vector3, Vector4};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Zeroable, bytemuck::Pod, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[allow(unused)]
impl Color {
    /// Create a color using r, g, b, a. All values go from 0-1
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Get the same color a different transparency/alpha.
    pub const fn with_alpha(&self, alpha: f32) -> Self {
        Self::new(self.r, self.g, self.b, alpha)
    }

    /// Create a color where RGA is a single number.
    pub const fn splat_alpha(v: f32, alpha: f32) -> Self {
        Self::new(v, v, v, alpha)
    }

    /// Create a color where RGA is a single number. (Alpha = 1)
    pub const fn splat(v: f32) -> Self {
        Self::splat_alpha(v, 1.0)
    }

    /// Linearly interpolate to another color using 0-1.
    pub fn lerp(self, other: Color, t: f32) -> Self {
        self + (other - self) * t
    }

    // Clamps the color from 0-1
    pub const fn clamped(&self) -> Self {
        Self::new(
            self.r.clamp(0.0, 1.0),
            self.g.clamp(0.0, 1.0),
            self.b.clamp(0.0, 1.0),
            self.a.clamp(0.0, 1.0),
        )
    }

    /// Create a color using RGBA. Each value goes from 0-255.
    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// Create a color using RGB. Each value goes from 0-255. (Alpha = 1)
    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
    }

    /// Create a color using hue-saturation-value-alpha. All values go from 0-1.
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        Self::from_hsva(h, s, v, 1.0)
    }

    /// Create a color using hue-saturation-value-alpha. All values go from 0-1.
    pub fn from_hsva(mut h: f32, s: f32, v: f32, a: f32) -> Self {
        h *= 360.0;
        let h = h.rem_euclid(360.0);
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h / 60.0 {
            h_prime if h_prime < 1.0 => (c, x, 0.0),
            h_prime if h_prime < 2.0 => (x, c, 0.0),
            h_prime if h_prime < 3.0 => (0.0, c, x),
            h_prime if h_prime < 4.0 => (0.0, x, c),
            h_prime if h_prime < 5.0 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::new(r + m, g + m, b + m, a)
    }

    /// Returns (h, s, v) all values from 0-1.
    pub fn to_hsv(&self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == self.r {
            60.0 * (((self.g - self.b) / delta).rem_euclid(6.0))
        } else if max == self.g {
            60.0 * (((self.b - self.r) / delta) + 2.0)
        } else {
            60.0 * (((self.r - self.g) / delta) + 4.0)
        };

        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;

        (h / 360.0, s, v)
    }

    /// Returns (h, s, v, a) all values from 0-1.
    pub fn to_hsva(&self) -> (f32, f32, f32, f32) {
        let (h, s, v) = self.to_hsv();
        (h, s, v, self.a)
    }

    /// Accepts "RRGGBB", "RGB", "RRGGBBAA", "RGBA", with or without leading '#'
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');

        let expand = |c: char| -> Option<u8> {
            let d = c.to_digit(16)? as u8;
            Some((d << 4) | d)
        };

        match hex.len() {
            3 => {
                let mut chars = hex.chars();
                let r = expand(chars.next()?)?;
                let g = expand(chars.next()?)?;
                let b = expand(chars.next()?)?;
                Some(Self::from_rgb8(r, g, b))
            }
            4 => {
                let mut chars = hex.chars();
                let r = expand(chars.next()?)?;
                let g = expand(chars.next()?)?;
                let b = expand(chars.next()?)?;
                let a = expand(chars.next()?)?;
                Some(Self::from_rgba8(r, g, b, a))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::from_rgb8(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::from_rgba8(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Returns "#RRGGBB"
    pub fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}",
            (self.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }

    /// Returns "#RRGGBBAA"
    pub fn to_hex_alpha(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            (self.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.a.clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }

    /// Returns a tuple of (r, g, b) from 0-255.
    pub const fn to_rgb(&self) -> (u8, u8, u8) {
        (
            (self.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }

    /// Returns a tuple of `(r, g, b, a)` from 0-255.
    pub const fn to_rgba(&self) -> (u8, u8, u8, u8) {
        (
            (self.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.a.clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }

    /// Create a color from a `Vector4<f32>` where `x -> r`, `y -> g`, `z -> b`, `w -> a`
    pub const fn from_vec4(vec: Vector4) -> Self {
        Self::new(vec.x, vec.y, vec.z, vec.w)
    }

    /// Create a `Vector4<f32>` from a color where `r -> x`, `g -> y`, `b -> z`, `a -> w`
    pub const fn to_vec4(&self) -> Vector4 {
        Vector4::new(self.r, self.g, self.b, self.a)
    }

    /// Create a color from a `Vector3<f32>` where `x -> r`, `y -> g`, `z -> b`
    pub const fn from_vec3(vec: Vector3) -> Self {
        Self::new(vec.x, vec.y, vec.z, 1.0)
    }

    /// Create a `Vector3<f32>` from a color where `r -> x`, `g -> y`, `b -> z`
    pub const fn to_vec3(&self) -> Vector3 {
        Vector3::new(self.r, self.g, self.b)
    }

    /// Convert color to an array of [r, g, b, a]
    pub const fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Convert color to a `wgpu::Color`
    pub const fn to_wgpu(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        c.to_array()
    }
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

impl Add<Color> for Color {
    type Output = Self;

    fn add(self, rhs: Color) -> Self::Output {
        Color::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl Sub<Color> for Color {
    type Output = Self;

    fn sub(self, rhs: Color) -> Self::Output {
        Color::new(
            self.r - rhs.r,
            self.g - rhs.g,
            self.b - rhs.b,
            self.a - rhs.a,
        )
    }
}

impl Mul<Color> for Color {
    type Output = Self;

    fn mul(self, rhs: Color) -> Self::Output {
        Color::new(
            self.r * rhs.r,
            self.g * rhs.g,
            self.b * rhs.b,
            self.a * rhs.a,
        )
    }
}

impl Div<Color> for Color {
    type Output = Self;

    fn div(self, rhs: Color) -> Self::Output {
        Color::new(
            self.r / rhs.r,
            self.g / rhs.g,
            self.b / rhs.b,
            self.a / rhs.a,
        )
    }
}

impl AddAssign<Color> for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.a += rhs.a;
    }
}

impl SubAssign<Color> for Color {
    fn sub_assign(&mut self, rhs: Color) {
        self.r -= rhs.r;
        self.g -= rhs.g;
        self.b -= rhs.b;
        self.a -= rhs.a;
    }
}

impl MulAssign<Color> for Color {
    fn mul_assign(&mut self, rhs: Color) {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
        self.a *= rhs.a;
    }
}

impl DivAssign<Color> for Color {
    fn div_assign(&mut self, rhs: Color) {
        self.r /= rhs.r;
        self.g /= rhs.g;
        self.b /= rhs.b;
        self.a /= rhs.a;
    }
}

// Scalar variants
impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Color::new(self.r * rhs, self.g * rhs, self.b * rhs, self.a * rhs)
    }
}

impl Div<f32> for Color {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Color::new(self.r / rhs, self.g / rhs, self.b / rhs, self.a / rhs)
    }
}

impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, rhs: f32) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
        self.a *= rhs;
    }
}

impl DivAssign<f32> for Color {
    fn div_assign(&mut self, rhs: f32) {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
        self.a /= rhs;
    }
}

macro_rules! define_colors {
    ($($name:ident = ($r:expr, $g:expr, $b:expr, $a:expr)),* $(,)?) => {
        impl Color {
            $(pub const $name: Self = Self::new($r, $g, $b, $a);)*
        }
        $(pub const $name: Color = Color::$name;)*
    };
}

define_colors! {
    TRANSPARENT = (0.0, 0.0, 0.0, 0.0),
    WHITE       = (1.0, 1.0, 1.0, 1.0),
    BLACK       = (0.0, 0.0, 0.0, 1.0),
    LIGHT_BLACK = (0.03, 0.03, 0.03, 1.0),

    GRAY        = (0.2, 0.2, 0.2, 1.0),
    DARK_GRAY   = (0.1, 0.1, 0.1, 1.0),
    LIGHT_GRAY  = (0.3, 0.3, 0.3, 1.0),

    RED         = (1.0, 0.03, 0.04, 1.0),
    DARK_RED    = (0.7, 0.03, 0.04, 1.0),
    LIGHT_RED   = (1.0, 0.2, 0.2, 1.0),

    GREEN       = (0.0, 1.0, 0.0, 1.0),
    DARK_GREEN  = (0.0, 0.3, 0.0, 1.0),
    LIGHT_GREEN = (0.4, 1.0, 0.3, 1.0),

    BLUE        = (0.03, 0.04, 1.0, 1.0),
    DARK_BLUE   = (0.02, 0.03, 0.7, 1.0),
    LIGHT_BLUE  = (0.2, 0.4, 1.0, 1.0),

    YELLOW      = (1.0, 1.0, 0.03, 1.0),
    DARK_YELLOW = (0.7, 0.7, 0.02, 1.0),
    LIGHT_YELLOW= (1.0, 1.0, 0.2, 1.0),

    MAGENTA       = (1.0, 0.03, 1.0, 1.0),
    DARK_MAGENTA  = (0.7, 0.02, 0.7, 1.0),
    LIGHT_MAGENTA = (1.0, 0.2, 1.0, 1.0),

    PURPLE       = (0.3, 0.03, 1.0, 1.0),
    DARK_PURPLE  = (0.2, 0.02, 0.7, 1.0),
    LIGHT_PURPLE = (0.45, 0.2, 1.0, 1.0),

    CYAN        = (0.03, 1.0, 1.0, 1.0),
    DARK_CYAN   = (0.02, 0.7, 0.7, 1.0),
    LIGHT_CYAN  = (0.5, 1.0, 1.0, 1.0),

    ORANGE      = (1.0, 0.3, 0.03, 1.0),
    DARK_ORANGE = (0.7, 0.2, 0.02, 1.0),
    LIGHT_ORANGE= (1.0, 0.45, 0.2, 1.0),
}
