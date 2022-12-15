use std::error::Error;
use std::fmt;
use std::sync::{Arc, RwLock};

pub type SampleType = f64;
pub const SAMPLE_RATE: SampleType = 44100f64;
pub const AUDIO_BUF_SIZE: usize = 1024;
pub type IoPort = Arc<RwLock<Option<SampleType>>>;
pub type ModuleResult<T> = std::result::Result<T, ModuleNotFoundError>;
pub type PortResult<T> = std::result::Result<T, PortNotFoundError>;

#[derive(Debug, Clone)]
pub struct ModuleNotFoundError;

impl Error for ModuleNotFoundError {}

impl fmt::Display for ModuleNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Module doesn't exist")
    }
}

#[derive(Debug, Clone)]
pub struct PortNotFoundError;

impl Error for PortNotFoundError {}

impl fmt::Display for PortNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Port doesn't exist")
    }
}
