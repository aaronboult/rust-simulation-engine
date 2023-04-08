use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window as WinitWindow, WindowBuilder},
};

pub mod camera;
pub mod context;
pub mod frame;
pub mod light;
pub mod pipeline;

use camera::Axis;
pub use context::Context;

#[cfg(target_arch = "wasm32")]
pub const WASM_ELEMENT_ID: &str = "wasm-canvas";

// Frame rate config
const FRAME_RATE_SHOW_DEFAULT: bool = true;

// Initial window config
const INITIAL_WIDTH: u32 = 450;
const INITIAL_HEIGHT: u32 = 400;
const INITIAL_TITLE: &str = "Simulation Engine";
const INITIAL_RESIZABLE: bool = true;

// Wrapper for the winit window to handle wasm32 specific stuff
pub struct Window {
    show_frame_rate: bool,
}

impl Window {
    pub fn new() -> Self {
        Self {
            show_frame_rate: FRAME_RATE_SHOW_DEFAULT,
        }
    }

    pub async fn run(mut self) {
        // Set up the window and event loop
        let event_loop = EventLoop::new();
        let winit_window = WindowBuilder::new().build(&event_loop).unwrap();

        // Append canvas to dom if wasm32
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| {
                    let wasm_element = document.get_element_by_id(WASM_ELEMENT_ID)?;
                    let canvas_element = web_sys::Element::from(winit_window.canvas());

                    wasm_element.append_child(&canvas_element).ok()?;

                    Some(())
                })
                .expect("Couldn't append canvas to document body");
        }

        winit_window.set_resizable(INITIAL_RESIZABLE);
        winit_window.set_inner_size(winit::dpi::LogicalSize::new(INITIAL_WIDTH, INITIAL_HEIGHT));
        Self::set_title(&winit_window, INITIAL_TITLE);

        // Create the context
        let mut context = Context::new(winit_window).await;

        context.init();

        log::info!("Starting mainloop");

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

                        // Keyboard input
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(keycode),
                                    ..
                                },
                            ..
                        } => {
                            match keycode {
                                // Toggle frame rate
                                VirtualKeyCode::F => {
                                    self.show_frame_rate = !self.show_frame_rate;
                                }
                                VirtualKeyCode::Up => {
                                    context.camera().set_up_axis(Axis::Y);
                                }
                                VirtualKeyCode::Left => {
                                    context.camera().set_up_axis(Axis::X);
                                }
                                VirtualKeyCode::Right => {
                                    context.camera().set_up_axis(Axis::Z);
                                }
                                _ => {}
                            }
                        }

                        // Resize
                        WindowEvent::Resized(physical_size) => {
                            context.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            context.resize(**new_inner_size);
                        }

                        _ => {
                            // log::info!("Unhandled event: {:?}", event);
                        }
                    }
                }
            }

            // Render
            Event::RedrawRequested(window_id) if window_id == context.window().id() => {
                context.update();

                match context.render() {
                    Ok(_) => {}

                    // Surface lost
                    Err(wgpu::SurfaceError::Lost) => context.resize(context.size()),

                    // Out of memory
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                    // Other error
                    Err(error) => log::error!("{:?}", error),
                }
            }
            Event::MainEventsCleared => {
                context.finalize();

                // let frame_rate = context.frame_rate();

                // Log frame rate
                // log::info!("FPS: {:.0}", frame_rate);

                // Update window title
                self.update_title(&context);
            }

            _ => {}
        });
    }

    fn update_title(&mut self, context: &Context) {
        if self.show_frame_rate {
            Self::set_title(
                context.window(),
                &format!(
                    "{} - {:.2} FPS - {:.2} AVG FPS",
                    INITIAL_TITLE,
                    context.frame_rate(),
                    context.average_frame_rate()
                ),
            );
        } else {
            Self::set_title(context.window(), INITIAL_TITLE);
        }
    }

    fn set_title(winit_window: &WinitWindow, title: &str) {
        winit_window.set_title(title);

        // Set document title if wasm32
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| {
                    document.set_title(title);

                    Some(())
                })
                .expect("Couldn't set document title");
        }
    }
}
