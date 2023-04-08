#[cfg(target_arch = "wasm32")]
use js_sys::Date;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[derive(Clone, Copy, Debug)]
pub struct Timestamp {
    #[cfg(target_arch = "wasm32")]
    start: f64,

    #[cfg(not(target_arch = "wasm32"))]
    start: Instant,

    is_empty: bool,
}

impl Timestamp {
    #[cfg(target_arch = "wasm32")]
    pub fn now() -> Self {
        Self {
            start: Date::new_0().get_time(),
            is_empty: false,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn now() -> Self {
        Self {
            start: Instant::now(),
            is_empty: false,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn elapsed(&self) -> f32 {
        if self.is_empty {
            return 0.0;
        }

        (Date::new_0().get_time() - self.start) as f32 / 1000.0
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn elapsed(&self) -> f32 {
        if self.is_empty {
            return 0.0;
        }

        self.start.elapsed().as_secs_f32()
    }

    pub fn empty() -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            start: Date::new_0().get_time(),

            #[cfg(not(target_arch = "wasm32"))]
            start: Instant::now(),

            is_empty: true,
        }
    }

    pub fn delta(&self, other: &Self) -> f32 {
        if self.is_empty || other.is_empty {
            return 0.0;
        }

        self.elapsed() - other.elapsed()
    }
}
