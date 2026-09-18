use crate::model::instance::Instance;
use crate::shapes::Shape;
use crate::shapes::circle::{CIRCLE_INDICES, CIRCLE_VERTICES};
use crate::shapes::color::Color;
use crate::shapes::rectangle::{RECT_INDICES, RECT_VERTICES};
use crate::shapes::triangle::{TRI_INDICES, TRI_VERTICES};
use cgmath::num_traits::real::Real;
use cgmath::{InnerSpace, MetricSpace, Quaternion, Rotation, Rotation3, Vector2, Vector3, Zero};
use wgpu::util::DeviceExt;
use wgpu::{Buffer, BufferUsages, Device, IndexFormat, RenderPass, util};

#[derive(Clone, Debug)]
pub struct DrawState {
    /// # Rectangle
    rect_vertex_buffer: Buffer,
    rect_index_buffer: Buffer,
    /// # Circle
    circle_vertex_buffer: Buffer,
    circle_index_buffer: Buffer,
    /// # Triangle
    triangle_vertex_buffer: Buffer,
    triangle_index_buffer: Buffer,

    queue: Vec<(Shape, Instance)>,
}

impl DrawState {
    pub fn new(device: &Device) -> Self {
        let (rect_vertex_buffer, rect_index_buffer) =
            Self::create_vertex_index_buffer(device, "Rectangle", RECT_VERTICES, RECT_INDICES);
        let (circle_vertex_buffer, circle_index_buffer) =
            Self::create_vertex_index_buffer(device, "Circle", CIRCLE_VERTICES, CIRCLE_INDICES);
        let (triangle_vertex_buffer, triangle_index_buffer) =
            Self::create_vertex_index_buffer(device, "Triangle", TRI_VERTICES, TRI_INDICES);

        Self {
            rect_vertex_buffer,
            rect_index_buffer,

            circle_vertex_buffer,
            circle_index_buffer,

            triangle_vertex_buffer,
            triangle_index_buffer,

            queue: Default::default(),
        }
    }

    pub fn create_vertex_index_buffer<A: bytemuck::Pod, B: bytemuck::Pod>(
        device: &Device,
        label: &str,
        vertices: &[A],
        indices: &[B],
    ) -> (Buffer, Buffer) {
        (
            device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some(&format!("{} Vertex Buffer", label)),
                contents: bytemuck::cast_slice(vertices),
                usage: BufferUsages::VERTEX,
            }),
            device.create_buffer_init(&util::BufferInitDescriptor {
                label: Some(&format!("{} Index Buffer", label)),
                contents: bytemuck::cast_slice(indices),
                usage: BufferUsages::INDEX,
            }),
        )
    }

    pub fn is_shape_queue_empty(&self, shape: Shape) -> bool {
        !self.queue.iter().any(|(x, _)| x == shape)
    }

    fn get_shape_queue(&self, shape: Shape) -> Vec<Instance> {
        self.queue
            .iter()
            .filter(|(x, _)| x == shape)
            .map(|(_, instance)| instance.clone())
            .collect()
    }

    fn get_instance_buffer(instances: &[Instance], device: &Device) -> Buffer {
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();

        device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: BufferUsages::VERTEX,
        })
    }

    pub fn set_shape_buffers(&self, device: &Device, pass: &mut RenderPass, shape: Shape) -> usize {
        let (vertex_buffer, index_buffer) = match shape {
            Shape::Rectangle => (&self.rect_vertex_buffer, &self.rect_index_buffer),
            Shape::Circle => (&self.circle_vertex_buffer, &self.circle_index_buffer),
            Shape::Triangle => (&self.triangle_vertex_buffer, &self.triangle_index_buffer),
        };

        let instances = self.get_shape_queue(shape);

        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, Self::get_instance_buffer(&instances, device).slice(..));
        pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);

        instances.len()
    }

    pub fn get_shape_indices(&self, shape: Shape) -> u32 {
        match shape {
            Shape::Rectangle => RECT_INDICES.len() as u32,
            Shape::Circle => CIRCLE_INDICES.len() as u32,
            Shape::Triangle => TRI_INDICES.len() as u32,
        }
    }
}

impl DrawState {
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn draw_rectangle(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.queue.push((
            Shape::Rectangle,
            Instance {
                position: Vector3::new(x, y, 0.0),
                size: Vector2::new(w, h),
                rotation: Quaternion::zero(),
                color,
                shape: Shape::Rectangle,
                tri_points: [Vector2::zero(), Vector2::zero(), Vector2::zero()],
            },
        ));
    }

    pub fn draw_rectangle_rotated(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rotation: f32,
        color: Color,
    ) {
        self.queue.push((
            Shape::Rectangle,
            Instance {
                position: Vector3::new(x, y, 0.0),
                size: Vector2::new(w, h),
                rotation: Quaternion::from_angle_z(cgmath::Rad(rotation)),
                color,
                shape: Shape::Rectangle,
                tri_points: [Vector2::zero(), Vector2::zero(), Vector2::zero()],
            },
        ));
    }

    pub fn draw_circle(&mut self, x: f32, y: f32, r: f32, color: Color) {
        self.queue.push((
            Shape::Circle,
            Instance {
                position: Vector3::new(x, y, 0.0),
                size: Vector2::new(r, r),
                rotation: Quaternion::zero(),
                color,
                shape: Shape::Circle,
                tri_points: [Vector2::zero(), Vector2::zero(), Vector2::zero()],
            },
        ));
    }

    pub fn draw_triangle(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        color: Color,
    ) {
        self.queue.push((
            Shape::Triangle,
            Instance {
                position: Vector3::new(0.0, 0.0, 0.0),
                size: Vector2::new(1.0, 1.0),
                rotation: Quaternion::zero(),
                color,
                shape: Shape::Triangle,
                tri_points: [
                    Vector2::new(x1, y1),
                    Vector2::new(x2, y2),
                    Vector2::new(x3, y3),
                ],
            },
        ));
    }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: Color) {
        let a = Vector2::new(x1, y1);
        let b = Vector2::new(x2, y2);
        let delta = b - a;
        let angle_radians = delta.y.atan2(delta.x);
        let perp_angle = angle_radians + std::f32::consts::FRAC_PI_2;;

        let half_thickness = thickness / 2.0;
        let offset_x = perp_angle.cos() * half_thickness;
        let offset_y = perp_angle.sin() * half_thickness;

        self.draw_rectangle_rotated(
            x1 - offset_x,
            y1 - offset_y,
            a.distance(b),
            thickness,
            angle_radians,
            color,
        );
    }

    pub fn draw_poly_line(&mut self, points: &[Vector2<f32>], thickness: f32, color: Color, closed: bool, rounded: bool) {
        for win in points.windows(2) {
            let (a, b) = (win[0], win[1]);

            if rounded {
                self.draw_circle(a.x, a.y, thickness, color);
            }
            self.draw_line(a.x, a.y, b.x, b.y, thickness, color);
        }

        if closed && let Some(a) = points.first() && let Some(b) = points.last() {
            self.draw_line(a.x, a.y, b.x, b.y, thickness, color);
        }

        if rounded && let Some(a) = points.last() {
            self.draw_circle(a.x, a.y, thickness, color);
        }
    }
}
