use cgmath::Vector3;

pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn to_vector3(&self) -> Vector3<f32> {
        match self {
            Axis::X => Vector3::unit_x(),
            Axis::Y => Vector3::unit_y(),
            Axis::Z => Vector3::unit_z(),
        }
    }
}
