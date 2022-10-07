use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use crate::types::{IoPort, SampleType};
use crate::control::Control;

/// An control
pub struct ControlKnob {
    /// A unique string used for identifying the module
    id: String,

    /// The control's output value
    out_value: IoPort,

    /// check if the control is focussed
    has_focus: AtomicBool,
}

impl ControlKnob {
    /// Create a new, unordered IoModule
    pub fn new(id: String) -> Self {
        let out_value = Arc::new(RwLock::new(None));
        let has_focus = AtomicBool::new(false);

        Self {
            id,
            out_value,
            has_focus,
        }
    }
}

impl Control for ControlKnob {
    /// Get a reference to the control's output port
    fn get_port_reference(&self) -> IoPort {
        self.out_value.clone()
    }

    /// Set the controls output value
    fn set_value(&self, new_value: Option<SampleType>) {
        if let Ok(mut value) = self.out_value.write() {
            *value = new_value;
        }
    }

    /// Receive and handle a control keys.
    /// For a control knob, controls relate to increasing or decreasing output
    /// TODO: Set increment value in struct
    /// TODO: Add other keys for fine-grained control
    fn recv_control_key(&self, key: char) {
        match key {
            'k' => {
                let next_value = match *self.out_value.read().unwrap() {
                    Some(val) => Some(val + 100f32),
                    None => None,
                };
                self.set_value(next_value);
            },
            'j' => {
                let next_value = match *self.out_value.read().unwrap() {
                    Some(val) => Some(val - 100f32),
                    None => None,
                };
                self.set_value(next_value);
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

