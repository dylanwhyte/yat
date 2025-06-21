use std::sync::{Arc, RwLock};

use crate::clock::Clock;
use crate::types::{IoPort,SampleType, PortResult, PortNotFoundError};
use crate::modules::io_module::IoModule;

/// A module that outputs it's input at a delayed time
pub struct Delay {
    /// A unique string used for identifying the module
    id: String,

    /// Order of the module in the chain, where 0 (zero) means skipped
    order: Option<u64>,

    input_ports: Vec<String>,

    output_ports: Vec<String>,

    /// The input audio signal
    in_audio: IoPort,

    /// The signal which informs the envelope generator that a note is
    /// active
    in_trigger: IoPort,

    /// The delay time (seconds)
    in_delay_time: IoPort,

    /// The modulated output signal of the envelope generator
    out_audio_out: IoPort,

    /// The current time for which a note has been active
    active_time: SampleType,

    /// The time at which a note was triggered
    trigger_time: SampleType,

    /// Time of the rack's clock
    clock: Arc<RwLock<Clock>>,
}

impl Delay {
    /// Create a new, unordered IoModule
    pub fn new(id: String, time: Arc<RwLock<Clock>>) -> Self {
        let order = None;
        let input_ports = vec!["amp".to_string(), "freq".to_string()];
        let output_ports = vec!["audio_out".to_string()];

        let in_audio = Arc::new(RwLock::new(None));
        let in_trigger = Arc::new(RwLock::new(None));
        let in_delay_time = Arc::new(RwLock::new(None));
        let out_audio_out = Arc::new(RwLock::new(None));

        let active_time = 0f64;
        let trigger_time = 0f64;

        Self {
            id,
            order,
            input_ports,
            output_ports,
            in_audio,
            in_trigger,
            in_attack,
            in_decay,
            in_sustain,
            in_release,
            out_audio_out,
            active_time,
            trigger_time,
            clock: time,
        }
    }

}

impl PartialEq for Delay {
    fn eq(&self, other: &Self) -> bool {
            self.id == other.id
    }
}

impl IoModule for Delay {
    /// Read inputs and populate outputs
    // TODO: Write this function
    fn process_inputs(&mut self) {


        self.active_time += clock.time_delta;

        let mut value = self.out_audio_out.write().expect("RwLock is poisoned");
        *value = Some(audio_out);
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
            "audio_in" => Some(self.in_audio.clone()),
            "trigger" => Some(self.in_trigger.clone()),
            "attack" => Some(self.in_attack.clone()),
            "decay" => Some(self.in_decay.clone()),
            "sustain" => Some(self.in_sustain.clone()),
            "release" => Some(self.in_release.clone()),
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
           "audio_in" => self.in_audio = out_port.clone(),
           "trigger" => self.in_trigger = out_port.clone(),
           "attack" => self.in_attack = out_port.clone(),
           "decay" => self.in_decay = out_port.clone(),
           "sustain" => self.in_sustain = out_port.clone(),
           "release" => self.in_release = out_port.clone(),
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

