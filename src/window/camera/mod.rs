use cgmath::{perspective, Deg, Matrix4, Point3, Vector3};

pub mod axis;
pub mod controller;
pub mod uniform;

pub use axis::Axis;

const CAMERA_EYE_DEFAULT: Point3<f32> = Point3::new(0.0, 1.0, 2.0); // 2 units back, 1 unit up
const CAMERA_ASPECT_DEFAULT: f32 = 16.0 / 9.0; // 16:9 aspect ratio
const CAMERA_FIELD_OF_VIEW_Y_DEFAULT: f32 = 45.0; // 45 degree field of view
const CAMERA_Z_NEAR_DEFAULT: f32 = 0.1; // 0.1 units in front of camera is near plane
const CAMERA_Z_FAR_DEFAULT: f32 = 100.0; // 100 units in front of camera is far plane
const CAMERA_TARGET_DEFAULT: Point3<f32> = Point3 {
    // Origin
    x: 0.0,
    y: 0.0,
    z: 0.0,
};
const CAMERA_UP_AXIS_DEFAULT: Axis = Axis::Y;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        // Point camera at target, oriented on up
        let view_matrix = Matrix4::look_at_rh(self.eye, self.target, self.up);

        // Calculate the projection matrix
        let projection_matrix = perspective(Deg(self.fov_y), self.aspect, self.z_near, self.z_far);

        OPENGL_TO_WGPU_MATRIX * projection_matrix * view_matrix
    }

    // Set aspect (width that can be f32, height that can be f32)
    // Needs to support u32 as f32
    pub fn set_aspect(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }

    pub fn set_up_axis(&mut self, up_axis: Axis) {
        self.up = up_axis.to_vector3();
    }

    pub fn translate(&mut self, translation: Vector3<f32>) {
        self.eye += translation;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: CAMERA_EYE_DEFAULT,
            target: CAMERA_TARGET_DEFAULT,
            up: CAMERA_UP_AXIS_DEFAULT.to_vector3(),
            aspect: CAMERA_ASPECT_DEFAULT,
            fov_y: CAMERA_FIELD_OF_VIEW_Y_DEFAULT,
            z_near: CAMERA_Z_NEAR_DEFAULT,
            z_far: CAMERA_Z_FAR_DEFAULT,
        }
    }
}
