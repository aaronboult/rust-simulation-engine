use cgmath::prelude::*;
use cgmath::{Deg, Matrix3, Matrix4, Quaternion, Vector3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    object: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl InstanceRaw {
    pub fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

// Instance
pub struct Instance {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    // _scale: Vector3<f32>,
}

impl Instance {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>) -> Self {
        Self { position, rotation }
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0)),
        }
    }
}

impl From<&Instance> for InstanceRaw {
    fn from(instance: &Instance) -> Self {
        InstanceRaw {
            object: (Matrix4::from_translation(instance.position)
                * Matrix4::from(instance.rotation))
            .into(),
            normal: Matrix3::from(instance.rotation).into(),
        }
    }
}
