use cpal::{self, Host, Device, SupportedStreamConfig};
use cpal::traits::{DeviceTrait, HostTrait};

pub struct CpalConfig {
    pub host: Host,
    pub device: Device,
    pub config: SupportedStreamConfig,
}

impl CpalConfig {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no device available");
        let config = device.default_output_config().unwrap();

        Self {
            host,
            device,
            config,
        }
    }
}
