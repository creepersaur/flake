use crate::camera::Camera;
use cgmath::Vector2;

pub struct Camera2D {
    pub position: Vector2<f32>,
    pub width: f32,
    pub height: f32,
    pub zoom: f32,
}

impl Camera for Camera2D {
    fn matrix(&self) -> cgmath::Matrix4<f32> {
        let left = self.position.x;
        let right = self.position.x + self.width / self.zoom;
        let top = self.position.y;
        let bottom = self.position.y + self.height / self.zoom;

        let (near, far) = (-1000.0, 1000.0);
        let proj = cgmath::ortho(left, right, bottom, top, near, far);

        // OpenGL clip z [-1,1] -> wgpu clip z [0,1]
        #[rustfmt::skip]
        let opengl_to_wgpu = cgmath::Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        );

        opengl_to_wgpu * proj
    }
}
