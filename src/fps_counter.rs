extern crate time;

use time::*;

pub struct FpsCounter {
    elapsed: Duration,
    n_frames: i32,
    fps: f32,
    update_period: Duration
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        FpsCounter {
            elapsed: Duration::zero(),
            n_frames: 0,
            fps: 0.0,
            update_period: Duration::milliseconds(1000)
        }
    }
    pub fn add_frame(&mut self, duration: Duration) {
        self.n_frames += 1;
        self.elapsed = self.elapsed + duration;
        if (self.elapsed >= self.update_period) {
            self.fps = 1000.0 * self.n_frames as f32 / (self.elapsed.num_milliseconds() as f32);
            self.elapsed = Duration::zero();
            self.n_frames = 0;
        }
        if self.fps < 10.0 {
            self.update_period = Duration::seconds(10);
        } else {
            self.update_period = Duration::seconds(1);
        }
    }
}

impl ToString for FpsCounter {
    fn to_string(&self) -> String {
        if self.fps < 10.0 {
            format!("{:.3} fps", self.fps)
        } else {
            format!("{:.0} fps", self.fps)
        }
    }
}
