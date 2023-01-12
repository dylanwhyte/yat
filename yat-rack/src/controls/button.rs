use std::sync::{Arc, RwLock};

use crate::{
    controls::control::Control,
    types::{IoPort, SampleType},
};

/// An control IoModule
pub struct Button {
    /// A unique string used for identifying the module
    id: String,

    /// The control's output value
    out_gate: IoPort,
}

impl Button {
    /// Create a new, unordered IoModule
    pub fn new(id: String) -> Self {
        let out_value = Arc::new(RwLock::new(None));

        Self {
            id,
            out_gate: out_value,
        }
    }
}

impl Control for Button {
    /// Get a reference to the control's output port
    fn get_port_reference(&self, port: &str) -> Option<IoPort> {
        match port {
            "gate" => Some(self.out_gate.clone()),
            _ => None,
        }
    }

    /// Set the controls output value
    fn set_value(&self, port: &str, new_value: Option<SampleType>) {
        if port == "gate" {
            let mut value = self.out_gate.write().expect("RwLock is poisoned");

            if let Some(new_value) = new_value {
                if new_value > 0f64 {
                    *value = Some(1f64);
                } else {
                    *value = Some(0f64);
                }
            }
        }
    }

    /// Receive and handle a control keys.
    /// For a button, the spacebar toggles the button on and off
    fn recv_control_key(&self, key: char) {
        if key == ' ' {
            // Toggle between on and off, using space
            let next_value = match *self.out_gate.read().expect("RwLock has been poisoned") {
                Some(current_val) => {
                    if current_val > 0f64 {
                        Some(0f64)
                    } else {
                        Some(1f64)
                    }
                }
                None => None,
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
