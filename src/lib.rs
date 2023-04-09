mod model;
mod resource;
mod texture;
mod time;
mod window;

use window::Window;

use async_std::task::block_on;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    // Toggle logging based on whether we are using webassembly or not
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    // Create and run window
    block_on(Window::new().run());
}
