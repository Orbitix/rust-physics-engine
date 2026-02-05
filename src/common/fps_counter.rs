const FPS_HISTORY_SIZE: usize = 60; // Number of frames to average over

pub struct SmoothedFps {
    pub history: [f32; FPS_HISTORY_SIZE],
    pub index: usize,
    pub sum: f32,
    pub count: usize,
}

impl SmoothedFps {
    pub fn new() -> Self {
        Self {
            history: [0.0; FPS_HISTORY_SIZE],
            index: 0,
            sum: 0.0,
            count: 0,
        }
    }

    pub fn update(&mut self, fps: f32) {
        if self.count < FPS_HISTORY_SIZE {
            self.count += 1;
        } else {
            // Subtract the value being replaced from the sum
            self.sum -= self.history[self.index];
        }

        // Add the new FPS value
        self.sum += fps;
        self.history[self.index] = fps;

        // Move to the next index, wrapping around if needed
        self.index = (self.index + 1) % FPS_HISTORY_SIZE;
    }

    pub fn get_average(&self) -> f32 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f32
        }
    }
}
