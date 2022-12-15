use crate::types::{SampleType, SAMPLE_RATE};
use std::sync::atomic::AtomicBool;

pub struct Clock {
    time: SampleType,
    pub time_delta: SampleType,
    running: AtomicBool,
}

impl Clock {
    pub fn new() -> Self {
        let time = 0f64;
        // FIXME: SAMPLE_RATE should be taken from CPAL config
        let time_delta = 1.0 / SAMPLE_RATE;
        let running = AtomicBool::new(false);

        Clock {
            time,
            time_delta,
            running,
        }
    }

    pub fn start_clock(&mut self) {
        *self.running.get_mut() = true;

        while *self.running.get_mut() {
            self.increment();
        }
    }

    pub fn stop_clock(&mut self) {
        *self.running.get_mut() = false;
    }

    pub fn reset_clock(&mut self) {
        self.time = 0.0;
    }

    pub fn get_time_ref(&self) -> SampleType {
        self.time
    }

    pub fn get_current_time(&self) -> Option<SampleType> {
        Some(self.time)
    }

    pub fn set_time(&mut self, new_time: SampleType) {
        self.time = new_time;
    }

    pub fn increment(&mut self) {
        if self.time >= 100_000f64 {
            self.time -= 100_000f64;
        } else {
            self.time += self.time_delta;
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}
