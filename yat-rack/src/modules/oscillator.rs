use std::sync::{Arc, RwLock};

use crate::types::{IoPort,SampleType, PortResult, PortNotFoundError};
use crate::modules::io_module::IoModule;

/// An oscillator IoModule
pub struct Oscillator {
    /// A unique string used for identifying the module
    id: String,

    /// Order of the module in the chain, where 0 (zero) means skipped
    order: Option<u64>,

    input_ports: Vec<String>,

    output_ports: Vec<String>,

    in_amp: IoPort,

    in_freq: IoPort,

    out_audio_out: IoPort,

    /// Time of the rack's clock
    time: Arc<RwLock<SampleType>>,
}

impl Oscillator {
    /// Create a new, unordered IoModule
    pub fn new(id: String, time: Arc<RwLock<SampleType>>) -> Self {
        let order = None;
        let input_ports = vec!["amp".to_string(), "freq".to_string()];
        let output_ports = vec!["audio_out".to_string()];

        let in_amp = Arc::new(RwLock::new(None));
        let in_freq = Arc::new(RwLock::new(None));
        let out_audio_out = Arc::new(RwLock::new(None));

        Self {
            id,
            order,
            input_ports,
            output_ports,
            in_amp,
            in_freq,
            out_audio_out,
            time,
        }
    }

}

impl PartialEq for Oscillator {
    fn eq(&self, other: &Self) -> bool {
            self.id == other.id
    }
}

impl IoModule for Oscillator {
    /// Read inputs and populate outputs
    fn process_inputs(&mut self) {
        let pi: SampleType = 3.14159265359;

        // FIXME: Add time to module
        let time = *self.time.read().unwrap();


        let amp = self.in_amp.read().unwrap().unwrap_or(0.5);

        let freq = self.in_freq.read().unwrap().unwrap_or(400.0);

        let audio_out = amp * (2.0 * pi * freq * time).sin();

        if let Ok(mut value) = self.out_audio_out.write() {
            *value = Some(audio_out);
        }
    }

    /// Return a module's ID
    fn get_id(&self) -> &String {
        &self.id
    }

    fn get_in_ports(&self) -> &Vec<String> {
        &self.input_ports
    }

    fn get_out_ports(&self) -> &Vec<String> {
        &self.output_ports
    }

    /// Return a reference to one of the module's input ports
    fn get_in_port_ref(&self, port_id: &str) -> Option<IoPort> {
        match port_id {
            "amp" => Some(self.in_amp.clone()),
            "freq" => Some(self.in_freq.clone()),
            _ => None,
        }
    }

    fn get_out_port_ref(&self, port_id: &str) -> Option<IoPort> {
        match port_id {
            "audio_out" => Some(self.out_audio_out.clone()),
            _ => None,
        }
    }

    /// Set the value of a module's input port
    fn set_in_port(&mut self, port_id: &str, out_port: IoPort) -> PortResult<String> {
        match port_id {
            "amp" => self.in_amp = out_port.clone(),
            "freq" => self.in_freq = out_port.clone(),
            _ => { return Err(PortNotFoundError) },
        }

        Ok(format!("{}: Set port {}\n", self.get_id(), port_id))
    }

    fn get_module_order(&self) -> Option<u64> {
        self.order
    }

    fn set_module_order(&mut self, new_order: Option<u64>) {
        self.order = new_order;
    }
}

