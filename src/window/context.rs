use std::collections::VecDeque;

use cgmath::prelude::*;
use cgmath::{Deg, Quaternion, Vector3};

use winit::{event::*, window::Window};

use bytemuck;
use cgmath;
use wgpu::{util::DeviceExt, CompositeAlphaMode};

use crate::{model, resource, texture, window};
use window::frame::Frame;
use window::pipeline::create_render_pipeline;

use camera::controller::CameraController;
use camera::uniform::CameraUniform;
use camera::Camera;
use light::LightUniform;
use model::instance::{Instance, InstanceRaw};
use model::{DrawLight, DrawModel, Vertex};
use window::{camera, light};

// Type alias for size
type WindowSize = winit::dpi::PhysicalSize<u32>;

const NUM_INSTANCES_PER_ROW: u32 = 10;
const SPACE_BETWEEN_MODELS: f32 = 3.0;

const MODEL_SHADER_STR: &'static str = include_str!("../shaders/shader.wgsl");
const LIGHT_SHADER_STR: &'static str = include_str!("../shaders/light.wgsl");

const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.3,
    a: 1.0,
};

// Frame rate config
const FRAME_BUFFER_LENGTH: usize = 128;
const FRAME_RATE_BUFFER_LENGTH: usize = 128;

const PRESENT_MODE_DEFAULT: wgpu::PresentMode = if cfg!(target_arch = "wasm32") {
    wgpu::PresentMode::AutoVsync
} else {
    // wgpu::PresentMode::AutoVsync
    wgpu::PresentMode::AutoNoVsync
};

const CAMERA_ROTATION_PER_SECOND: f32 = 30.0;

pub struct Context {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: WindowSize,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    light_uniform: LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    object_model: model::Model,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    frame_buffer: VecDeque<Frame>,
    frame_current: Frame,
    frame_rate_buffer: VecDeque<f32>,
}

impl Context {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // Get GPU handle and allow all gpu apis
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // Get the surface
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // Next create a device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        // Configure surface
        let surface_capabilities = surface.get_capabilities(&adapter);

        // Assumes sRGB surface texture
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|format| format.describe().srgb)
            .next()
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PRESENT_MODE_DEFAULT,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        // Create depth texture
        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "DepthTexture");

        // Create diffuse texture
        const DIFFUSE_BYTES: &[u8] = include_bytes!("../textures/happy-tree.png");

        // Create texture bind group
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // normal map
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("TextureBindGroupLayout"),
            });

        // Lighting
        let light_uniform = LightUniform::new([2.0, 2.0, 2.0], [1.0, 1.0, 1.0]);

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("LightBuffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        // Camera
        let mut camera = Camera::default();

        // Move camera back 4 units and up 2 units
        camera.translate([0.0, 2.0, 4.0].into());

        let camera_uniform = CameraUniform::from(&camera);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("CameraBuffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("CameraBindGroupLayout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("CameraBindGroup"),
        });

        let camera_controller = Default::default();

        // Create instances - TEMPORARY
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN_MODELS * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN_MODELS * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = Vector3 { x, y: 0.0, z };

                    let rotation = if position.is_zero() {
                        Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0))
                    } else {
                        Quaternion::from_axis_angle(position.normalize(), Deg(45.0))
                    };

                    Instance::new(position, rotation)
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(InstanceRaw::from).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("InstanceBuffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create render pipeline
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("RenderPipelineLayout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("NormalShader"),
                source: wgpu::ShaderSource::Wgsl(MODEL_SHADER_STR.into()),
            };

            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[
                    model::ModelVertex::get_buffer_layout(),
                    InstanceRaw::get_buffer_layout(),
                ],
                shader,
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("LightPipelineLayout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("LightShader"),
                source: wgpu::ShaderSource::Wgsl(LIGHT_SHADER_STR.into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::get_buffer_layout()],
                shader,
            )
        };

        let object_model =
            resource::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            depth_texture,
            light_uniform,
            light_buffer,
            light_bind_group,
            light_render_pipeline,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            object_model,
            instances,
            instance_buffer,
            frame_buffer: VecDeque::with_capacity(FRAME_BUFFER_LENGTH),
            frame_current: Frame::empty(),
            frame_rate_buffer: VecDeque::with_capacity(FRAME_RATE_BUFFER_LENGTH),
        }
    }

    pub fn size(&self) -> WindowSize {
        self.size
    }

    pub fn resize(&mut self, new_size: WindowSize) {
        // 1x1 minimum size to prevent wgpu panics - needs refactoring into config
        if new_size.width > 1 && new_size.height > 1 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.camera
                .set_aspect(self.config.width as f32, self.config.height as f32);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "DepthTexture");
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        let should_return = self.camera_controller.process_events(event);

        if should_return {
            return true;
        }

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                // Update light color using cursor position
                let x = (position.x / self.size.width as f64) as f32;
                let y = (position.y / self.size.height as f64) as f32;

                self.light_uniform.set_color([x, y, 0.5]);

                true
            }

            _ => false,
        }
    }

    pub fn update(&mut self) {
        let last_frame = self.frame_buffer.back().unwrap();

        self.camera_controller
            .update_camera(&mut self.camera, last_frame);

        self.camera_uniform
            .update_view_projection_matrix(&self.camera);

        // Update camera buffer
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        // Rotate light
        let old_position: Vector3<_> = self.light_uniform.position.into();
        let rotation_angle = Deg(CAMERA_ROTATION_PER_SECOND) * last_frame.delta_time();

        self.light_uniform.set_position_into(
            Quaternion::from_axis_angle(Vector3::unit_y().into(), rotation_angle) * old_position,
        );

        // Update light buffer
        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[self.light_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Get frame
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("RenderEncoder"),
            });

        {
            // Render pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("RenderPass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(BACKGROUND_COLOR),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // Set vertex buffer
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            // Draw light
            render_pass.set_pipeline(&self.light_render_pipeline);
            render_pass.draw_light_model(
                &self.object_model,
                &self.camera_bind_group,
                &self.light_bind_group,
            );

            // Draw object
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw_model_instanced(
                &self.object_model,
                0..self.instances.len() as u32,
                &self.camera_bind_group,
                &self.light_bind_group,
            );
        }

        // Submit command buffer
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn frame_rate(&self) -> f32 {
        // Mean of frame times
        let frame_time_sum = self
            .frame_buffer
            .iter()
            .fold(0.0, |acc, frame| acc + frame.frame_time_s());

        self.frame_buffer.len() as f32 / ((frame_time_sum * 100.0).round() / 100.0)
    }

    pub fn average_frame_rate(&self) -> f32 {
        // Mean of frame rate buffer
        let frame_rate_sum = self
            .frame_rate_buffer
            .iter()
            .fold(0.0, |acc, frame_rate| acc + frame_rate);

        frame_rate_sum / self.frame_rate_buffer.len() as f32
    }

    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn init(&mut self) {
        self.frame_current.begin();
    }

    pub fn finalize(&mut self) {
        self.window.request_redraw();

        self.frame_current.end();
        self.frame_buffer.push_back(self.frame_current);

        if self.frame_buffer.len() > FRAME_BUFFER_LENGTH {
            self.frame_buffer.pop_front();
        }

        self.frame_current = Frame::new();

        self.frame_rate_buffer.push_back(self.frame_rate());

        if self.frame_rate_buffer.len() > FRAME_RATE_BUFFER_LENGTH {
            self.frame_rate_buffer.pop_front();
        }
    }
}
