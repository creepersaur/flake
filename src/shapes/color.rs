#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[allow(unused)]
impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn to_wgpu(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
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
    RED         = (1.0, 0.03, 0.04, 1.0),
    GREEN       = (0.0, 1.0, 0.0, 1.0),
    BLUE        = (0.03, 0.04, 1.0, 1.0),
    YELLOW      = (1.0, 1.0, 0.03, 1.0),
    MAGENTA     = (1.0, 0.03, 1.0, 1.0),
    PURPLE      = (0.3, 0.03, 1.0, 1.0),
    CYAN        = (0.03, 1.0, 1.0, 1.0),
    ORANGE      = (1.0, 0.3, 0.03, 1.0),
    GRAY       = (0.3, 0.3, 0.3, 1.0),
}
