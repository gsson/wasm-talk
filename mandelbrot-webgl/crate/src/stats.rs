pub struct FrameStats {
    frames: usize,

    next_log_timestamp: usize,
    last_frame_timestamp: usize,

    frame_times: [usize; 32],
}

impl FrameStats {
    pub fn new(now: f64) -> FrameStats {
        let unow = now as usize;
        FrameStats {
            frames: 0,
            next_log_timestamp: unow + 2000,
            last_frame_timestamp: unow,
            frame_times: [0; 32],
        }
    }

    pub fn frame<F>(&mut self, now: f64, f: F)
    where
        F: FnOnce(usize, f32),
    {
        let unow = now as usize;

        let frame_time = unow - self.last_frame_timestamp;
        self.frame_times[self.frames & 31] = frame_time;

        self.frames += 1;
        self.last_frame_timestamp = unow;

        if unow > self.next_log_timestamp {
            self.next_log_timestamp = unow + 2000;
            let frame_time_sum: usize = self.frame_times.iter().sum();
            let frame_time_average: f32 = frame_time_sum as f32 / 32.0;
            let frames_per_second = 1000.0f32 / frame_time_average;
            f(self.frames, frames_per_second)
        }
    }
}
