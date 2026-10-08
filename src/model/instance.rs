use crate::draw_state::shapes::Shape;
use crate::draw_state::shapes::color::Color;
use cgmath::{Vector2, Vector3, Vector4, Zero};

pub(crate) const FULL_UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

pub(crate) struct Instance {
    pub position: Vector3<f32>,
    pub rotation: f32, // radians
    pub size: Vector2<f32>,
    pub radius: Vector4<f32>,
    pub color: Color,
    pub shape: Shape,
    pub tri_points: [Vector2<f32>; 3],
    pub uv_rect: [f32; 4],
}

impl Instance {
    pub fn new(shape: Shape, x: f32, y: f32, w: f32, h: f32, color: Color) -> Self {
        Self {
            position: Vector3::new(x, y, 0.0),
            rotation: 0.0,
            size: Vector2::new(w, h),
            radius: Vector4::zero(),
            color,
            shape,
            tri_points: [Vector2::zero(); 3],
            uv_rect: FULL_UV,
        }
    }

    pub fn update_with_z_offset(mut self, z_offset: f32) -> Self {
        self.position.z = z_offset;
        self
    }

    pub fn to_raw(&self) -> InstanceRaw {
        let c = self
            .color
            .to_array()
            .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
        InstanceRaw {
            pos: self.position.into(),
            rotation: self.rotation,
            size: self.size.into(),
            radius: [
                self.radius.x.clamp(0.0, self.size.x.min(self.size.y) * 0.5),
                self.radius.y.clamp(0.0, self.size.x.min(self.size.y) * 0.5),
                self.radius.z.clamp(0.0, self.size.x.min(self.size.y) * 0.5),
                self.radius.w.clamp(0.0, self.size.x.min(self.size.y) * 0.5),
            ],
            color: c,
            shape: self.shape as u32,
            tri_points: self.tri_points.map(Into::into),
            uv_rect: self.uv_rect,
        }
    }
}
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct InstanceRaw {
    pos: [f32; 3],
    rotation: f32,
    size: [f32; 2],
    radius: [f32; 4],
    color: [u8; 4],
    shape: u32,
    tri_points: [[f32; 2]; 3],
    uv_rect: [f32; 4],
}

impl InstanceRaw {
    const ATTRS: [wgpu::VertexAttribute; 10] = wgpu::vertex_attr_array![
        5 => Float32x3, 6 => Float32, 7 => Float32x2, 8 => Float32x4,
        9 => Unorm8x4, 10 => Uint32,
        11 => Float32x2, 12 => Float32x2, 13 => Float32x2,
        14 => Float32x4,
    ];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRS,
        }
    }
}
