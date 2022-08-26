//use std::collections::{HashMap, HashSet};
use hashbrown::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::types::{IoPort,SampleType};
use crate::io_module_trait::IoModuleTrait;

/// An oscillator IoModule
pub struct Oscillator {
    /// A unique string used for identifying the module
    id: String,

    /// Order of the module in the chain, where 0 (zero) means skipped
    order: Option<u64>,

    input_ports: Vec<String>,

    in_amp: IoPort,

    in_freq: IoPort,

    out_audio_out: IoPort,

    /// The module's input ports
    in_ports: HashMap<String, IoPort>,

    /// The module's output ports
    out_ports: HashMap<String, IoPort>,

    /// Time of the rack's clock
    time: Arc<RwLock<SampleType>>,
}

impl Oscillator {
    /// Create a new, unordered IoModule
    pub fn new(id: String, time: Arc<RwLock<SampleType>>) -> Self {
        let order = None;
        let mut in_ports: HashMap<String, IoPort> = HashMap::new();
        in_ports.insert("amp".to_string(), Arc::new(RwLock::new(None)));
        in_ports.insert("freq".to_string(), Arc::new(RwLock::new(None)));

        let mut out_ports: HashMap<String, IoPort> = HashMap::new();
        out_ports.insert("audio_out".to_string(), Arc::new(RwLock::new(None)));

        let input_ports = vec!["amp".to_string(), "freq".to_string()];

        let in_amp = Arc::new(RwLock::new(None));
        let in_freq = Arc::new(RwLock::new(None));
        let out_audio_out = Arc::new(RwLock::new(None));

        Self {
            id,
            order,
            input_ports,
            in_amp,
            in_freq,
            out_audio_out,
            in_ports,
            out_ports,
            time,
        }
    }

}

impl PartialEq for Oscillator {
    fn eq(&self, other: &Self) -> bool {
            self.id == other.id
    }
}

impl IoModuleTrait for Oscillator {
    /// Read inputs and populate outputs
    fn process_inputs(&mut self) {
        let pi: SampleType = 3.14159265359;

        // FIXME: Add time to module
        let time = *self.time.read().unwrap();

        //let amp = self.read_in_port_value("amp").unwrap_or(0.5);
        //let freq = self.read_in_port_value("freq").unwrap_or(400.0);

        let amp = self.in_freq.read().unwrap().unwrap_or(0.5);
        //println!("amplitude {}", amp);

        let freq = self.in_freq.read().unwrap().unwrap_or(400.0);
        //println!("freq {}", freq);


        let audio_out = amp * (2.0 * pi * freq * time).sin();

        self.write_out_port_value("audio_out", Some(audio_out));
    }

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

    fn get_in_ports(&self) -> &Vec<String> {
        &self.input_ports
    }

    /// Return a reference to the module's input ports
    fn get_in_port_ref(&self, port_id: &str) -> Option<IoPort> {
        match port_id {
            "amp" => Some(self.in_amp.clone()),
            "freq" => Some(self.in_freq.clone()),
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
            "amp" => { println!("setting amp");
                self.in_amp = out_port.clone(); }
            "freq" => { println!("setting freq");
                self.in_freq = out_port.clone(); }
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
            "audio_out" => Some(self.out_audio_out.clone()),
            _ => None,
        }

        //if let Some(out_port) = self.out_ports.get(port_id) {
            //Some(out_port.to_owned().clone())
        //} else {
            //None
        //}
    }

    fn write_out_port_value(&self, out_port_label: &str, new_value: Option<SampleType>) {
        match out_port_label {
            "audio_out" => { if let Ok(mut value) = self.out_audio_out.write() {
                *value = new_value;
            }}
            _ => (),
        }
        //if let Some(out_port) = self.out_ports.get(out_port_label) {
            //if let Ok(mut value) = out_port.write() {
                //*value = new_value;
            //}
        //}
    }

    fn get_module_order(&self) -> Option<u64> {
        self.order
    }

    fn set_module_order(&mut self, new_order: Option<u64>) {
        self.order = new_order;
    }
}

