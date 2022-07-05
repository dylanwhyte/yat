use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::types::{IoPort,SampleType};
use crate::io_module_trait::IoModuleTrait;

/// An oscillator IoModule
pub struct Oscillator {
    /// A unique string used for identifying the module
    id: String,

    /// Order of the module in the chain, where 0 (zero) means skipped
    order: Option<u64>,

    /// The module's input ports
    in_ports: HashMap<String, IoPort>,

    /// The module's output ports
    out_ports: HashMap<String, IoPort>,

    /// Process inputs and write outputs
    process_fn: fn(&Oscillator, time: f64),
}

impl Oscillator {
    /// Create a new, unordered IoModule
    pub fn new(id: String, process_fn: fn(&Oscillator, time: f64)) -> Self {
        let order = None;
        let mut in_ports: HashMap<String, IoPort> = HashMap::new();
        in_ports.insert("amp".to_string(), Arc::new(RwLock::new(None)));
        in_ports.insert("freq".to_string(), Arc::new(RwLock::new(None)));

        let mut out_ports: HashMap<String, IoPort> = HashMap::new();
        out_ports.insert("audio_out".to_string(), Arc::new(RwLock::new(None)));

        Self {
            id,
            order,
            in_ports,
            out_ports,
            process_fn,
        }
    }

    /// Read inputs and populate outputs
    pub fn process_inputs(&self, time: f64) {
        (self.process_fn)(self, time);

        //let pi: SampleType = 3.14159265359;

        //let amp = self.read_in_port_value("amp").unwrap();
        //let freq = self.read_in_port_value("freq").unwrap();
        //let audio_out = amp * (2.0 * pi * freq * time).sin();

        //self.write_out_port_value("audio_out", Some(audio_out));

    }
}

impl PartialEq for Oscillator {
    fn eq(&self, other: &Self) -> bool {
            self.id == other.id
    }
}

impl IoModuleTrait for Oscillator {
    /// Return a module's ID
    fn get_id(&self) -> &String {
        &self.id
    }

    /// Add an input or output port to the module
    fn create_port(&mut self, port_type: &str, port_name: &str) {
        if port_type.eq("in") {
            self.in_ports.insert(port_name.to_string(), Arc::new(RwLock::new(None)));
        } else if port_type.eq("out") {
            self.out_ports.insert(port_name.to_string(), Arc::new(RwLock::new(None)));
        }
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


