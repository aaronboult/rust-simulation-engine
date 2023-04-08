use crate::time::Timestamp;

#[derive(Debug, Copy, Clone)]
pub struct Frame {
    start_time: Timestamp,
    end_time: Timestamp,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            start_time: Timestamp::now(),
            end_time: Timestamp::empty(),
        }
    }

    pub fn empty() -> Self {
        Self {
            start_time: Timestamp::empty(),
            end_time: Timestamp::empty(),
        }
    }

    pub fn begin(&mut self) {
        self.start_time = Timestamp::now();
    }

    pub fn end(&mut self) {
        self.end_time = Timestamp::now();
    }

    pub fn frame_time_s(&self) -> f32 {
        self.start_time.delta(&self.end_time)
    }

    pub fn delta_time(&self) -> f32 {
        self.start_time.elapsed()
    }
}
