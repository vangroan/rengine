//! Non-essential utilities

const FPS_COUNTER_WINDOW_SIZE: usize = 60;

pub struct FpsCounter {
    /// Sliding window of timer durations for
    /// past frames, stored as seconds.
    frames: [f32; FPS_COUNTER_WINDOW_SIZE],

    /// Current position in frame duration window
    cursor: usize,
}

impl FpsCounter {
    pub fn new() -> Self {
        FpsCounter::default()
    }

    /// Records the delta time for a frame.
    pub fn add(&mut self, duration: &::std::time::Duration) {
        let millis = duration.as_millis();
        self.frames[self.cursor] = millis as f32 / 1000.0;
        self.cursor = (self.cursor + 1) % FPS_COUNTER_WINDOW_SIZE;
    }

    /// Calculates the frames per second for the past
    /// frame window.
    pub fn fps(&self) -> f32 {
        assert!(FPS_COUNTER_WINDOW_SIZE != 0);

        let total = self.frames.iter().fold(0.0, |acc, x| acc + x);
        let average_dt = total / FPS_COUNTER_WINDOW_SIZE as f32;

        if average_dt != 0.0 {
            1.0 / average_dt
        } else {
            0.0
        }
    }
}

impl Default for FpsCounter {
    fn default() -> Self {
        FpsCounter {
            frames: [0.0; FPS_COUNTER_WINDOW_SIZE],
            cursor: 0,
        }
    }
}
