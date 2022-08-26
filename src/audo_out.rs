//use std::collections::{HashMap, HashSet};
use hashbrown::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{self, sync_channel, SyncSender, Receiver};
use std::thread;

use cpal::{Data, Sample, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::types::{IoPort,SampleType};
use crate::io_module_trait::IoModuleTrait;

/// An oscillator IoModule
pub struct AudioOut {
    /// A unique string used for identifying the module
    id: String,

    /// Order of the module in the chain, where 0 (zero) means skipped
    order: Option<u64>,

    /// The module's input ports
    in_ports: HashMap<String, IoPort>,

    audio_tx: mpsc::SyncSender<SampleType>,

    /// The module's output ports
    out_ports: HashMap<String, IoPort>,

}

impl AudioOut {
    /// Create a new, unordered IoModule
    //pub fn new(id: String, audio_out_ref: IoPort) -> (Self, Receiver<SampleType>) {
    pub fn new(id: String) -> (Self, Receiver<SampleType>) {
        let order = None;
        let mut in_ports: HashMap<String, IoPort> = HashMap::new();
        in_ports.insert("audio_in".to_string(), Arc::new(RwLock::new(None)));

        let mut out_ports: HashMap<String, IoPort> = HashMap::new();
        //out_ports.insert(String::from("audio_out"), audio_out_ref);

        let (audio_tx, audio_rx) = mpsc::sync_channel(44100);

        let audio_out = Self {
            id,
            order,
            in_ports,
            audio_tx,
            out_ports,
        };

        //audio_out.generate_audio_thread();

        (audio_out, audio_rx)
    }

}

impl PartialEq for AudioOut {
    fn eq(&self, other: &Self) -> bool {
            self.id == other.id
    }
}

impl IoModuleTrait for AudioOut {
    fn process_inputs(&mut self) {
        let audio_in = self.read_in_port_value("audio_in");


        if let Some(audio_in) = audio_in {
            //self.write_out_port_value("audio_out", Some(audio_in));
            self.audio_tx.send(audio_in).unwrap();
        }

    }

    /// Return a module's ID
    fn get_id(&self) -> &String {
        &self.id
    }

    /// Add an input or output port to the module
    fn create_port(&mut self, port_type: &str, port_name: &str) {
        //if port_type.eq("in") {
            //self.in_ports.insert(port_name.to_string(), Arc::new(RwLock::new(None)));
        //} else if port_type.eq("out") {
            //self.out_ports.insert(port_name.to_string(), Arc::new(RwLock::new(None)));
        //}
    }

    /// Return a reference to the module's input ports
    fn get_in_ports_ref(&self) -> &HashMap<String, IoPort> {
        &self.in_ports
    }

    /// Return a reference to the module's input ports
    fn get_in_port_ref(&self, port_id: &str) -> Option<IoPort> {
        if let Some(in_port) = self.in_ports.get(port_id) {
            Some(in_port.to_owned().clone())
        } else {
            None
        }
    }

    /// Set the value of a module's input port
    // TODO: Handle non-existent port case
    fn set_in_port(&mut self, port_id: &str, out_port: IoPort) {
        if self.in_ports.contains_key(port_id) {
            self.in_ports.insert(port_id.to_string(), out_port.clone());
        }
    }

    /// Get the actual value at a port
    fn read_in_port_value(&self, in_port_label: &str) -> Option<SampleType> {
        if let Some(in_port) = self.in_ports.get(in_port_label) {
            *in_port.to_owned().read().unwrap()
        } else {
            None
        }
    }

    fn get_out_ports_ref(&self) -> &HashMap<String, IoPort> {
        &self.out_ports
    }

    fn get_out_port_ref(&self, port_id: &str) -> Option<IoPort> {
        if let Some(out_port) = self.out_ports.get(port_id) {
            Some(out_port.to_owned().clone())
        } else {
            None
        }
    }

    fn write_out_port_value(&self, out_port_label: &str, new_value: Option<SampleType>) {
        if let Some(out_port) = self.out_ports.get(out_port_label) {
            if let Ok(mut value) = out_port.write() {
                *value = new_value;
            }
        }
    }

    fn get_module_order(&self) -> Option<u64> {
        self.order
    }

    fn set_module_order(&mut self, new_order: Option<u64>) {
        self.order = new_order;
    }
}

