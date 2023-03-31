use std::sync::{RwLock, Weak};

use crate::controls::control::Control;
use crate::types::SampleType;
use crate::out_port::OutPort;

/// An control IoModule
pub struct Button {
    /// A unique string used for identifying the module
    id: String,

    /// The control's output value
    out_gate: OutPort,
}

impl Button {
    /// Create a new, unordered IoModule
    pub fn new(id: String) -> Self {
        let out_value = OutPort::new("gate".into());

        Self {
            id,
            out_gate: out_value,
        }
    }
}

impl Control for Button {
    /// Get a reference to the control's output port
    fn get_port_reference(&self, port: &str)
        -> Option<Weak<RwLock<Option<SampleType>>>> {
        match port {
            "gate" => Some(self.out_gate.get_ref()),
            _ => None,
        }
    }

    /// Set the controls output value
    fn set_value(&self, port: &str, new_value: SampleType) {
        if port == "gate" {
            if new_value > 0.0 {
                self.out_gate.set_value(1.0);
            } else {
                self.out_gate.set_value(0.0);
            }
        }
    }

    /// Receive and handle a control keys.
    /// For a button, the spacebar toggles the button on and off
    fn recv_control_key(&self, key: char) {
        if key == ' ' {
            // Toggle between on and off, using space
            let next_value = match self.out_gate.get_ref().upgrade() {
                Some(val) => {
                    let current_val = val.read().expect("RwLock poisoned").unwrap_or(0.0);
                    if current_val > 0f64 {
                        0f64
                    } else {
                        1f64
                    }
                }
                None => 1f64,
            };
            self.set_value("gate", next_value);
        }
    }
}

impl PartialEq for Button {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
