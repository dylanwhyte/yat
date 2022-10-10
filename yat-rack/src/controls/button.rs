use std::sync::{Arc, RwLock};

use crate::{types::{IoPort, SampleType}, controls::control::Control};

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
        match port {
            "gate" => {
                if let Ok(mut value) = self.out_gate.write() {
                    if let Some(new_value) = new_value {
                        if new_value > 0f32 {
                            *value = Some(1f32);
                        } else {
                            *value = Some(0f32);
                        }
                    }
                }
            },
            _ => {},

        }
   }

    /// Receive and handle a control keys.
    /// For a button, the spacebar toggles the button on and off
    fn recv_control_key(&self, key: char) {
        match key {
            // Toggle between on and off, using space
            ' ' => {
                let next_value = match *self.out_gate.read().unwrap() {
                    Some(current_val) => {
                        if current_val > 0f32 {
                            Some(0f32)
                        } else {
                            Some(1f32)
                        }
                    },
                    None => None,
                };
                self.set_value("gate", next_value);
            },
            _ => (),
        }
    }

}

impl PartialEq for Button {
    fn eq(&self, other: &Self) -> bool {
            self.id == other.id
    }
}

