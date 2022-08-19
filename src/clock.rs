use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;
use crate::types::{SampleType, SAMPLE_RATE};

pub struct Clock {
    pub time: Arc<RwLock<SampleType>>,
    pub time_delta: SampleType,
    running: AtomicBool,
}

impl Clock {
    pub fn new() -> Self {
        let time = Arc::new(RwLock::new(0.0));
        let time_delta = 1.0 / (SAMPLE_RATE as SampleType);
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
        if let Ok(mut time) = self.time.write() {
            *time = 0.0;
        }
    }

    pub fn get_time_ref(&self) -> Arc<RwLock<SampleType>> {
        self.time.clone()
    }

    pub fn get_current_time(&self) -> Option<SampleType> {
        if let Ok(time) = self.time.read() {
            Some(*time)
        } else {
            None
        }
    }

    pub fn set_time(&mut self, new_time: SampleType) {
        if let Ok(mut time) = self.time.write() {
            *time = new_time;
        }
    }

    pub fn increment(&self) {
        if let Ok(mut current_time) = self.time.write() {
            if *current_time >= 1.0 {
                *current_time -= 1.0;
            } else {
                *current_time += self.time_delta;
            }
        }
    }
}


