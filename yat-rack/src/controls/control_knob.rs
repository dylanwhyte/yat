use std::sync::{Arc, RwLock};

use crate::types::{IoPort, SampleType};
use crate::controls::control::Control;

/// An control
pub struct ControlKnob {
    /// A unique string used for identifying the module
    id: String,

    /// The control's output value
    out_value: IoPort,
}

impl ControlKnob {
    /// Create a new, unordered IoModule
    pub fn new(id: String) -> Self {
        let out_value = Arc::new(RwLock::new(None));

        Self {
            id,
            out_value,
        }
    }
}

impl Control for ControlKnob {
    /// Get a reference to the control's output port
    fn get_port_reference(&self, port_id: &str) -> Option<IoPort> {
        match port_id {
            "value" => Some(self.out_value.clone()),
            _ => None,
        }
    }

    /// Set the controls output value
    fn set_value(&self, port_id: &str, new_value: Option<SampleType>) {
        match port_id {
            "value" => {
                let mut value = self.out_value.write().expect("RwLock is poisoned");
                *value = new_value;
            },
            _ => {},
        }
    }

    /// Receive and handle a control keys.
    /// For a control knob, controls relate to increasing or decreasing output
    /// TODO: Set increment value in struct
    /// TODO: Add other keys for fine-grained control
    fn recv_control_key(&self, key: char) {
        match key {
            'k' => {
                let next_value = match *self.out_value.read().expect("RwLock is poisoned") {
                    Some(val) => Some(val + 100f64),
                    None => Some(0f64),
                };
                self.set_value("value", next_value);
            },
            'j' => {
                let next_value = match *self.out_value.read().expect("RwLock is poisoned") {
                    Some(val) => Some(val - 100f64),
                    None => Some(1f64),
                };
                self.set_value("value", next_value);
            },
            _ => (),
        }
    }
}

impl PartialEq for ControlKnob {
    fn eq(&self, other: &Self) -> bool {
            self.id == other.id
    }
}

