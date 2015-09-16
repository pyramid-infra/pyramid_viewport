extern crate time;

use time::*;

pub struct FpsCounter {
    elapsed: Duration,
    n_frames: i32,
    n_total_frames: i32,
    fps: f32
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        FpsCounter {
            elapsed: Duration::zero(),
            n_frames: 0,
            n_total_frames: 0,
            fps: 0.0
        }
    }
    pub fn add_frame(&mut self, duration: Duration) {
        self.n_frames += 1;
        self.n_total_frames += 1;
        self.elapsed = self.elapsed + duration;
        if self.elapsed >= Duration::seconds(1) {
            self.fps = 1000.0 * self.n_frames as f32 / (self.elapsed.num_milliseconds() as f32);
            self.elapsed = Duration::zero();
            self.n_frames = 0;
        }
    }
}

impl ToString for FpsCounter {
    fn to_string(&self) -> String {
        let tot = match self.n_total_frames < 1000 {
            true => format!("#{}", self.n_total_frames),
            false => "".to_string()
        };
        if self.fps < 10.0 {
            format!("{:.3} fps, {:.3} ms/f {}", self.fps, 1000.0/self.fps, tot)
        } else {
            format!("{:.0} fps, {:.5} ms/f {}", self.fps, 1000.0/self.fps, tot)
        }
    }
}
