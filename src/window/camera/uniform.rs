use cgmath::prelude::*;
use cgmath::Matrix4;

use crate::window::camera::Camera;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    view_projection_matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_projection_matrix: Matrix4::identity().into(),
        }
    }

    pub fn update_view_projection_matrix(&mut self, camera: &Camera) {
        // We're using Vector4 because of the uniforms 16 byte spacing requirement
        self.view_position = camera.eye.to_homogeneous().into();
        self.view_projection_matrix =
            (OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
    }
}

// CameraUniform From Camera
impl From<&Camera> for CameraUniform {
    fn from(camera: &Camera) -> Self {
        let mut camera_uniform = Self::new();

        camera_uniform.update_view_projection_matrix(camera);

        camera_uniform
    }
}
