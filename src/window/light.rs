#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

impl LightUniform {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            _padding: 0,
            color,
            _padding2: 0,
        }
    }

    pub fn set_color<T>(&mut self, color: T)
    where
        T: Into<[f32; 3]>,
    {
        self.color = color.into();
    }

    pub fn set_position_into<T>(&mut self, position: T)
    where
        T: Into<[f32; 3]>,
    {
        self.position = position.into();
    }
}
