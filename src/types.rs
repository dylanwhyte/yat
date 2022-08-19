#![allow(dead_code)]
use std::sync::{Arc, RwLock};
use std::fmt;

pub type SampleType = f32;
pub const SAMPLE_RATE: u32 = 44100;
pub type IoPort = Arc<RwLock<Option<SampleType>>>;
pub type ModuleResult<T> = std::result::Result<T, ModuleNotFoundError>;

#[derive(Debug, Clone)]
pub struct ModuleNotFoundError;

impl fmt::Display for ModuleNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Module doesn't exist")
    }
}


