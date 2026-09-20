use crate::model::instance::InstanceRaw;
use wgpu::util::{BufferInitDescriptor, DeviceExt, RenderEncoder};
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, IndexFormat, Queue, RenderPass};

pub mod circle;
pub mod color;
pub mod polygon;
pub mod rectangle;
pub mod triangle;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Rectangle = 0,
    Circle = 1,
    Triangle = 2,
}

impl PartialEq<Shape> for &Shape {
    fn eq(&self, other: &Shape) -> bool {
        other == *self
    }
}

/// # Shape Buffer

#[derive(Clone, Debug)]
pub struct ShapeBuffer {
    pub shape: Shape,
    label: &'static str,
    instance_buffer: Option<Buffer>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    capacity: usize, // instance count
    count: usize,
}

impl ShapeBuffer {
    pub fn new<A: bytemuck::Pod, B: bytemuck::Pod>(
        device: &Device,
        shape: Shape,
        label: &'static str,
        vertices: &[A],
        indices: &[B],
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX,
        });

        Self {
            shape,
            label,
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            instance_buffer: None,
            capacity: 0,
            count: 0,
        }
    }

    fn alloc(&self, device: &Device) -> Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(self.label),
            size: (self.capacity * size_of::<InstanceRaw>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn upload(&mut self, device: &Device, queue: &Queue, data: &[InstanceRaw]) {
        self.count = data.len();
        if data.is_empty() {
            return;
        }
        if data.len() > self.capacity {
            self.capacity = data.len().next_power_of_two();
            self.instance_buffer = Some(self.alloc(device));
        }
        queue.write_buffer(
            self.instance_buffer.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(data),
        );
    }

    pub fn draw<'a>(&'a self, pass: &mut RenderPass<'a>) {
        let Some(inst) = &self.instance_buffer else {
            return;
        };
        if self.count == 0 {
            return;
        }
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(
            1,
            inst.slice(..(self.count * size_of::<InstanceRaw>()) as u64),
        );
        pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_count, 0, 0..self.count as u32);
    }
}
