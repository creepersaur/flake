use crate::draw_state::shapes::Shape;
use crate::draw_state::shapes::color::Color;
use cgmath::Vector2;

#[derive(Clone, Debug)]
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub size: cgmath::Vector2<f32>,
    pub color: Color,
    pub shape: Shape,
    pub tri_points: [Vector2<f32>; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 4],
    shape: u32,
    tri_points: [[f32; 2]; 3],
}

impl Instance {
    pub fn update_with_z_offset(&mut self, z_offset: f32) -> &mut Self {
        self.position.z += z_offset;
        self
    }
    
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation)
                * cgmath::Matrix4::from_nonuniform_scale(self.size.x, self.size.y, 1.0))
            .into(),
            color: self.color.to_array(),
            shape: self.shape as u32,
            tri_points: [
                self.tri_points[0].into(),
                self.tri_points[1].into(),
                self.tri_points[2].into(),
            ],
        }
    }
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Shape
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 20]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Uint32,
                },
                // Tri point 0
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 20]>() as wgpu::BufferAddress
                        + size_of::<u32>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Tri point 1
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 20]>() as wgpu::BufferAddress
                        + size_of::<u32>() as wgpu::BufferAddress
                        + size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Tri point 2
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 20]>() as wgpu::BufferAddress
                        + size_of::<u32>() as wgpu::BufferAddress
                        + size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 13,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
