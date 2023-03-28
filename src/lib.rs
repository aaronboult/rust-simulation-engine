use cgmath::prelude::*;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use bytemuck;
use cgmath;
use wgpu::util::DeviceExt;

mod texture;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
pub const WASM_ELEMENT_ID: &str = "wasm-example";

const INITIAL_WIDTH: u32 = 450;
const INITIAL_HEIGHT: u32 = 400;
const INITIAL_TITLE: &str = "Simulation Engine";
const INITIAL_RESIZABLE: bool = false;
const BACKGROUND_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.3,
    a: 1.0,
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

const INSTANCE_SHADER_LOCATION: u32 = 5; // 5-8 inclusive

// Type alias for size
type WindowSize = winit::dpi::PhysicalSize<u32>;

// Vertex
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    texture_coordinates: [f32; 2],
}

impl Vertex {
    const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::VERTEX_ATTRIBUTES,
        }
    }
}

// Polygon vertices
const _POLYGON_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        texture_coordinates: [0.4131759, 0.00759614],
    },
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        texture_coordinates: [0.0048659444, 0.43041354],
    },
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        texture_coordinates: [0.28081453, 0.949397],
    },
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        texture_coordinates: [0.85967, 0.84732914],
    },
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        texture_coordinates: [0.9414737, 0.2652641],
    },
];

// Polygon indicies
const _POLYGON_INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

// Cube vertices using Ccw winding and indexing
const _CUBE_VERTICES: &[Vertex] = &[
    // Top
    Vertex {
        position: [-0.5, 0.5, -0.5],
        texture_coordinates: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        texture_coordinates: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        texture_coordinates: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        texture_coordinates: [0.0, 1.0],
    },
    // Bottom
    Vertex {
        position: [-0.5, -0.5, -0.5],
        texture_coordinates: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        texture_coordinates: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        texture_coordinates: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        texture_coordinates: [1.0, 0.0],
    },
    // Left
    Vertex {
        position: [-0.5, 0.5, 0.5],
        texture_coordinates: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        texture_coordinates: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        texture_coordinates: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        texture_coordinates: [0.0, 1.0],
    },
    // Right
    Vertex {
        position: [0.5, 0.5, 0.5],
        texture_coordinates: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        texture_coordinates: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        texture_coordinates: [0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        texture_coordinates: [1.0, 0.0],
    },
    // Front
    Vertex {
        position: [0.5, 0.5, 0.5],
        texture_coordinates: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        texture_coordinates: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        texture_coordinates: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        texture_coordinates: [1.0, 1.0],
    },
    // Back
    Vertex {
        position: [0.5, 0.5, -0.5],
        texture_coordinates: [0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        texture_coordinates: [1.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        texture_coordinates: [1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        texture_coordinates: [0.0, 0.0],
    },
];

// Cube indicies
const _CUBE_INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, // Top
    4, 5, 6, 6, 7, 4, // Bottom
    8, 9, 10, 10, 11, 8, // Left
    12, 13, 14, 14, 15, 12, // Right
    16, 17, 18, 18, 19, 16, // Front
    20, 21, 22, 22, 23, 20, // Back
];

// Instance Raw
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    object: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn get_buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: INSTANCE_SHADER_LOCATION,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: INSTANCE_SHADER_LOCATION + 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: INSTANCE_SHADER_LOCATION + 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: INSTANCE_SHADER_LOCATION + 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

// Instance
struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    // _scale: cgmath::Vector3<f32>,
}

impl From<&Instance> for InstanceRaw {
    fn from(instance: &Instance) -> Self {
        InstanceRaw {
            object: (cgmath::Matrix4::from_translation(instance.position)
                * cgmath::Matrix4::from(instance.rotation))
            .into(),
        }
    }
}

// Camera
struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fov_y: f32,
    z_near: f32,
    z_far: f32,
}

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // Point camera at target, oriented on up
        let view_matrix = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        // Calculate the projection matrix
        let projection_matrix = cgmath::perspective(
            cgmath::Deg(self.fov_y),
            self.aspect,
            self.z_near,
            self.z_far,
        );

        OPENGL_TO_WGPU_MATRIX * projection_matrix * view_matrix
    }
}

// Camera Uniform
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection_matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_projection_matrix: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_projection_matrix(&mut self, camera: &Camera) {
        self.view_projection_matrix = camera.build_view_projection_matrix().into();
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

struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the fowrard/backward is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and eye so
            // that it doesn't change. The eye therefore still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}

// Context
struct Context {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: WindowSize,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    diffuse_bind_group: wgpu::BindGroup,
    _diffuse_texture: texture::Texture,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    background_color: wgpu::Color,
}

impl Context {
    async fn new(window: Window) -> Self {
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
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        // Load texture
        const DIFFUSE_BYTES: &[u8] = include_bytes!("textures/happy-tree.png");

        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, DIFFUSE_BYTES, "DiffuseTexture").unwrap();

        // Create bind group
        let diffuse_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("DiffuseBindGroupLayout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &diffuse_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("DiffuseBindGroup"),
        });

        // Camera
        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fov_y: 45.0,
            z_near: 0.1,
            z_far: 100.0,
        };

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
                    visibility: wgpu::ShaderStages::VERTEX,
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

        let camera_controller = CameraController::new(0.2);

        // Create render pipeline
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("VertexBuffer"),
            contents: bytemuck::cast_slice(_CUBE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create index buffer
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("IndexBuffer"),
            contents: bytemuck::cast_slice(_CUBE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create instances - TEMPORARY
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    } - INSTANCE_DISPLACEMENT;

                    let rotation = if position.is_zero() {
                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can effect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(InstanceRaw::from).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("InstanceBuffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("RenderPipelineLayout"),
                bind_group_layouts: &[&diffuse_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("RenderPipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::get_buffer_layout(),
                    InstanceRaw::get_buffer_layout(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),

                // Options other than fill require Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,

                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,

                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            index_count: _CUBE_INDICES.len() as u32,
            diffuse_bind_group,
            _diffuse_texture: diffuse_texture,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            instances,
            instance_buffer,
            background_color: BACKGROUND_COLOR,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: WindowSize) {
        // 1x1 minimum size to prevent wgpu panics - needs refactoring into config
        if new_size.width > 1 && new_size.height > 1 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        let should_return = self.camera_controller.process_events(event);

        if should_return {
            return true;
        }

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                // Update background color using cursor position
                let x = position.x / self.size.width as f64;
                let y = position.y / self.size.height as f64;

                self.background_color = wgpu::Color {
                    r: x as f64,
                    g: y as f64,
                    b: 0.5,
                    a: 1.0,
                };

                true
            }

            _ => false,
        }
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);

        self.camera_uniform
            .update_view_projection_matrix(&self.camera);

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                        load: wgpu::LoadOp::Clear(self.background_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // Set pipeline
            render_pass.set_pipeline(&self.render_pipeline);

            // Set bind group
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            // Set vertex buffer
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            // Set index buffer
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Draw 3 vertices with 1 instance
            render_pass.draw_indexed(0..self.index_count, 0, 0..self.instances.len() as _);
        }

        // Submit command buffer
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    // Toggle logging based on whether we are using webassembly or not
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    // Set up the window and event loop
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    window.set_title(INITIAL_TITLE);
    window.set_resizable(INITIAL_RESIZABLE);
    window.set_inner_size(winit::dpi::LogicalSize::new(INITIAL_WIDTH, INITIAL_HEIGHT));

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(INITIAL_WIDTH, INITIAL_HEIGHT));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| {
                let wasm_element = document.get_element_by_id(WASM_ELEMENT_ID)?;
                let canvas_element = web_sys::Element::from(window.canvas());
                wasm_element.append_child(&canvas_element).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // Create the context
    let mut context = Context::new(window).await;

    // Run the event loop
    event_loop.run(move |event, _, control_flow| match event {
        // Handle window events
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == context.window().id() => {
            if !context.input(event) {
                match event {
                    // Close or escape key pressed
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,

                    // Resize
                    WindowEvent::Resized(physical_size) => {
                        context.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        context.resize(**new_inner_size);
                    }

                    _ => {}
                }
            }
        }

        // Render
        Event::RedrawRequested(window_id) if window_id == context.window().id() => {
            context.update();

            match context.render() {
                Ok(_) => {}

                // Surface lost
                Err(wgpu::SurfaceError::Lost) => context.resize(context.size),

                // Out of memory
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                // Other error
                Err(error) => eprintln!("{:?}", error),
            }
        }
        Event::MainEventsCleared => {
            context.window().request_redraw();
        }

        _ => {}
    });
}
