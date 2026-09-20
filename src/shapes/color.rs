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

    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Self = Self::new(1.0, 0.03, 0.04, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.03, 0.04, 1.0, 1.0);
    pub const YELLOW: Self = Self::new(1.0, 1.0, 0.03, 1.0);
    pub const MAGENTA: Self = Self::new(1.0, 0.03, 1.0, 1.0);
    pub const PURPLE: Self = Self::new(0.3,0.03, 1.0, 1.0);
    pub const CYAN: Self = Self::new(0.03, 1.0, 1.0, 1.0);
    pub const ORANGE: Self = Self::new(1.0, 0.3, 0.03, 1.0);
}