extern crate time;

use time::*;

pub struct FpsCounter {
    elapsed: Duration,
    n_frames: i32,
    fps: f32
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        FpsCounter {
            elapsed: Duration::zero(),
            n_frames: 0,
            fps: 0.0
        }
    }
    pub fn add_frame(&mut self, duration: Duration) {
        self.n_frames += 1;
        self.elapsed = self.elapsed + duration;
        if (self.elapsed.num_milliseconds() >= 1000) {
            self.fps = 1000.0 * self.n_frames as f32 / (self.elapsed.num_milliseconds() as f32);
            self.elapsed = Duration::zero();
            self.n_frames = 0;
        }
    }
    pub fn fps(&self) -> f32 {
        self.fps
    }
}
