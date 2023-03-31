use std::sync::{RwLock, Weak};

use crate::modules::io_module::IoModule;
use crate::types::{PortNotFoundError, PortResult, SampleType, SAMPLE_RATE};
use crate::in_port::InPort;
use crate::out_port::OutPort;

/// An oscillator IoModule
pub struct Oscillator {
    /// A unique string used for identifying the module
    id: String,

    /// Order of the module in the chain, where 0 (zero) means skipped
    order: Option<u64>,

    input_ports: Vec<String>,

    output_ports: Vec<String>,

    in_amp: InPort,

    in_freq: InPort,

    out_audio_out: OutPort,

    /// Value for phase acucumulator
    phase: SampleType,
}

impl Oscillator {
    /// Create a new, unordered IoModule
    pub fn new(id: String) -> Self {
        let order = None;
        let input_ports = vec!["amp".to_string(), "freq".to_string()];
        let output_ports = vec!["audio_out".to_string()];

        let in_amp = InPort::new("amp".into(), 0.0, 1.0, 0.5);
        let in_freq = InPort::new("freq".into(), 0.0, 20_000.0, 1000.0);
        let out_audio_out = OutPort::new("audio_out".into());
        let phase = 0f64;

        Self {
            id,
            order,
            input_ports,
            output_ports,
            in_amp,
            in_freq,
            out_audio_out,
            phase,
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
        let pi = std::f64::consts::PI;

        let amp = self.in_amp.get_value();

        let freq = self.in_freq.get_value();

        self.phase += (2.0 * pi * freq) / SAMPLE_RATE;
        let audio_out = amp * self.phase.sin();

        self.out_audio_out.set_value(audio_out);
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
    fn has_port_with_id(&self, port_id: &str) -> bool {
        match port_id {
            "amp" | "freq"  => true,
            _ => false,
        }
    }

    fn get_out_port_ref(&self, port_id: &str) -> Option<&OutPort> {
        match port_id {
            "audio_out" => Some(&self.out_audio_out),
            _ => None,
        }
    }

    /// Set the value of a module's input port
    fn set_in_port(&mut self, port_id: &str, out_port_ref: Weak<RwLock<Option<SampleType>>>) -> PortResult<String> {
        match port_id {
            "amp" => self.in_amp.set_value(out_port_ref),
            "freq" => self.in_freq.set_value(out_port_ref),
            _ => return Err(PortNotFoundError),
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
