//use std::collections::{HashMap, HashSet};
use hashbrown::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{self, sync_channel, SyncSender, Receiver};
use std::thread;

use cpal::{Data, Sample, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::types::{AUDIO_BUF_SIZE, IoPort, SampleType};
use crate::io_module_trait::IoModuleTrait;

/// An oscillator IoModule
pub struct AudioOut {
    /// A unique string used for identifying the module
    id: String,

    /// Order of the module in the chain, where 0 (zero) means skipped
    order: Option<u64>,

    input_ports: Vec<String>,

    in_audio_in: IoPort,

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

        let input_ports = vec!["audio_in".to_string()];

        let in_audio_in = Arc::new(RwLock::new(None));

        let out_ports: HashMap<String, IoPort> = HashMap::new();
        //out_ports.insert(String::from("audio_out"), audio_out_ref);

        let (audio_tx, audio_rx) = mpsc::sync_channel(AUDIO_BUF_SIZE);

        let audio_out = Self {
            id,
            order,
            input_ports,
            in_audio_in,
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

        //let audio_in = self.read_in_port_value("audio_in");

        let audio_in = *self.in_audio_in.read().unwrap();

        self.audio_tx.send(audio_in.unwrap()).unwrap();


    }

    /// Return a module's ID
    fn get_id(&self) -> &String {
        &self.id
    }

    /// Return a reference to the module's input ports
    fn get_in_ports_ref(&self) -> &HashMap<String, IoPort> {
        &self.in_ports
    }


    fn get_in_ports(&self) -> &Vec<String> {
        &self.input_ports
    }

    /// Return a reference to the module's input ports
    fn get_in_port_ref(&self, port_id: &str) -> Option<IoPort> {
        match port_id {
            "audio_in" => Some(self.in_audio_in.clone()),
            _ => None,
        }
        //if let Some(in_port) = self.in_ports.get(port_id) {
            //Some(in_port.to_owned().clone())
        //} else {
            //None
        //}
    }

    /// Set the value of a module's input port
    // TODO: Handle non-existent port case
    fn set_in_port(&mut self, port_id: &str, out_port: IoPort) {
        match port_id {
            "audio_in" => {self.in_audio_in = out_port.clone() }
            _ => (),
        }
        //if self.in_ports.contains_key(port_id) {
            //self.in_ports.insert(port_id.to_string(), out_port.clone());
        //}
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
        match port_id {
            //"audio_out" => Some(self.out_audio_out.clone()),
            _ => None,
        }
        //if let Some(out_port) = self.out_ports.get(port_id) {
            //Some(out_port.to_owned().clone())
        //} else {
            //None
        //}
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

