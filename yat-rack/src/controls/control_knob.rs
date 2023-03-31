use std::sync::{RwLock, Weak};

use crate::controls::control::Control;
use crate::out_port::OutPort;
use crate::types::SampleType;

/// An control
pub struct ControlKnob {
    /// A unique string used for identifying the module
    id: String,

    /// The control's output value
    out_value: OutPort,
}

impl ControlKnob {
    /// Create a new, unordered IoModule
    pub fn new(id: String) -> Self {
        let out_value = OutPort::new("value".into());

        Self {
            id,
            out_value,
        }
    }
}

impl Control for ControlKnob {
    /// Get a reference to the control's output port
    fn get_port_reference(&self, port_id: &str)
        -> Option<Weak<RwLock<Option<SampleType>>>> {
        match port_id {
            "value" => Some(self.out_value.get_ref()),
            _ => None,
        }
    }

    /// Set the controls output value
    fn set_value(&self, port_id: &str, new_value: SampleType) {
        if port_id == "value" {
            self.out_value.set_value(new_value);
        }
    }

    /// Receive and handle a control keys.
    /// For a control knob, controls relate to increasing or decreasing output
    /// TODO: Set increment value in struct
    /// TODO: Add other keys for fine-grained control
    fn recv_control_key(&self, key: char) {
        match key {
            'k' => {
                let next_value = match self.out_value.get_ref().upgrade() {
                    Some(val) => {
                        let val = val.read().expect("RwLock poisoned").unwrap_or(0.0);
                        val + 100f64
                    },
                    None => 0f64,
                };
                self.set_value("value", next_value);
            }
            'j' => {
                let next_value = match self.out_value.get_ref().upgrade() {
                    Some(val) => {
                        let val = val.read().expect("RwLock poisoned").unwrap_or(0.0);
                        val - 100f64
                    },
                    None => 0f64,
                };
                self.set_value("value", next_value);
            }
            _ => (),
        }
    }
}

impl PartialEq for ControlKnob {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
