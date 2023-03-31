use std::sync::mpsc::{self, Receiver};
use std::sync::{RwLock, Weak};

use crate::in_port::InPort;
use crate::out_port::OutPort;
use crate::modules::io_module::IoModule;
use crate::types::{PortNotFoundError, PortResult, SampleType, AUDIO_BUF_SIZE};

/// An exit point from a Rack, e.g. for audio output
pub struct Output {
    /// A unique string used for identifying the module
    id: String,

    /// Order of the module in the chain, where 0 (zero) means skipped
    order: Option<u64>,

    input_ports: Vec<String>,

    output_ports: Vec<String>,

    in_signal_in: InPort,

    /// A channel for sending data from the rack's chain to outside the rack
    out_signal_tx: mpsc::SyncSender<SampleType>,
}

impl Output {
    /// Create a new, unordered IoModule
    //pub fn new(id: String, audio_out_ref: IoPort) -> (Self, Receiver<SampleType>) {
    pub fn new(id: String) -> (Self, Receiver<SampleType>) {
        let order = None;
        let input_ports = vec!["signal_in".to_string()];
        let output_ports = vec!["signal_out".to_string()];

        let in_signal_in = InPort::new("signal_in".into(), -1.0, 1.0, 0.0);

        let (out_signal_tx, signal_rx) = mpsc::sync_channel(AUDIO_BUF_SIZE);

        let output = Self {
            id,
            order,
            input_ports,
            output_ports,
            in_signal_in,
            out_signal_tx,
        };

        (output, signal_rx)
    }
}

impl PartialEq for Output {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl IoModule for Output {
    fn process_inputs(&mut self) {
        let signal_in = self.in_signal_in.get_value();

        self.out_signal_tx.send(signal_in).unwrap();
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

    /// Return a reference to the module's input ports
    fn has_port_with_id(&self, port_id: &str) -> bool {
        match port_id {
            "signal_in" => true,
            _ => false,
        }
    }

    fn get_out_port_ref(&self, _port_id: &str) -> Option<&OutPort> {
        None
    }

    /// Set the value of a module's input port
    fn set_in_port(&mut self, port_id: &str, out_port_ref: Weak<RwLock<Option<SampleType>>>) -> PortResult<String> {
        match port_id {
            "signal_in" => self.in_signal_in.set_value(out_port_ref),
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
