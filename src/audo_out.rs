use std::sync::{Arc, RwLock};
use std::sync::mpsc::{self, Receiver};

use crate::types::{AUDIO_BUF_SIZE, IoPort, PortResult, PortNotFoundError, SampleType};
use crate::io_module::IoModule;

/// An oscillator IoModule
pub struct AudioOut {
    /// A unique string used for identifying the module
    id: String,

    /// Order of the module in the chain, where 0 (zero) means skipped
    order: Option<u64>,

    input_ports: Vec<String>,

    output_ports: Vec<String>,

    in_audio_in: IoPort,

    audio_tx: mpsc::SyncSender<SampleType>,
}

impl AudioOut {
    /// Create a new, unordered IoModule
    //pub fn new(id: String, audio_out_ref: IoPort) -> (Self, Receiver<SampleType>) {
    pub fn new(id: String) -> (Self, Receiver<SampleType>) {
        let order = None;
        let input_ports = vec!["audio_in".to_string()];
        let output_ports = vec![];

        let in_audio_in = Arc::new(RwLock::new(None));

        let (audio_tx, audio_rx) = mpsc::sync_channel(AUDIO_BUF_SIZE);

        let audio_out = Self {
            id,
            order,
            input_ports,
            output_ports,
            in_audio_in,
            audio_tx,
        };

        (audio_out, audio_rx)
    }

}

impl PartialEq for AudioOut {
    fn eq(&self, other: &Self) -> bool {
            self.id == other.id
    }
}

impl IoModule for AudioOut {
    fn process_inputs(&mut self) {
        let audio_in = *self.in_audio_in.read().unwrap();

        self.audio_tx.send(audio_in.unwrap()).unwrap();
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
    fn get_in_port_ref(&self, port_id: &str) -> Option<IoPort> {
        match port_id {
            "audio_in" => Some(self.in_audio_in.clone()),
            _ => None,
        }
    }

    fn get_out_port_ref(&self, port_id: &str) -> Option<IoPort> {
        match port_id {
            //"audio_out" => Some(self.out_audio_out.clone()),
            _ => None,
        }
    }

    /// Set the value of a module's input port
    fn set_in_port(&mut self, port_id: &str, out_port: IoPort) -> PortResult<String> {
        match port_id {
            "audio_in" => {
                self.in_audio_in = out_port.clone();
            }
            _ => { return  Err(PortNotFoundError); },
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

