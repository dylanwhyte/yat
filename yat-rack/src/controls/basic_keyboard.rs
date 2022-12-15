use std::sync::{Arc, RwLock};

use crate::{types::{IoPort, SampleType}, controls::control::Control};

/// An control IoModule
pub struct BasicKeyboard {
    /// A unique string used for identifying the module
    id: String,

    /// A gate signal to communicate when a note is activated
    /// and deactivated
    out_gate: IoPort,

    /// The pitch representation of the note that is pressed
    out_pitch: IoPort,
}

impl BasicKeyboard {
    /// Create a new BasicKeyboard
    pub fn new(id: String) -> Self {
        let out_gate = Arc::new(RwLock::new(Some(0f64)));
        let out_pitch = Arc::new(RwLock::new(Some(0f64)));

        Self {
            id,
            out_gate,
            out_pitch,
        }
    }
}

impl Control for BasicKeyboard {
    /// Get a reference to the control's output port
    fn get_port_reference(&self, port: &str) -> Option<IoPort> {
        match port {
            "gate" => Some(self.out_gate.clone()),
            "pitch" => Some(self.out_pitch.clone()),
            _ => None,
        }
    }

    /// Set the controls output value
    fn set_value(&self, port: &str, new_value: Option<SampleType>) {
        match port {
            "gate" => {
                let mut value = self.out_gate.write().expect("RwLock is poisoned");
                *value = new_value;
            },
            "pitch" => {
                let mut value = self.out_pitch.write().expect("RwLock is poisoned");
                *value = new_value;
            },
            _ => {},
        }
    }

    /// Receive and handle a control keys.
    /// For a button, the spacebar toggles the button on and off
    fn recv_control_key(&self, key: char) {
        match key {
            // a4
            'a' => {
                self.set_value("pitch", Some(440f64));
            }
            // a#4
            'w' => {
                self.set_value("pitch", Some(466.16f64));
            }
            // b4
            's' => {
                self.set_value("pitch", Some(493.88f64));
            }
            // c5
            'd' => {
                self.set_value("pitch", Some(523.25f64));
            }
            // c#5
            'r' => {
                self.set_value("pitch", Some(554.37f64));
            }
            // d5
            'f' => {
                self.set_value("pitch", Some(587.33f64));
            }
            // d#5
            't' => {
                self.set_value("pitch", Some(622.25f64));
            }
            // e5
            'g' => {
                self.set_value("pitch", Some(659.26f64));
            }
            // f5
            'h' => {
                self.set_value("pitch", Some(698.46f64));
            }
            // f#5
            'u' => {
                self.set_value("pitch", Some(0f64));
            }
            // g5
            'j' => {
                self.set_value("pitch", Some(0f64));
            }
            // g#5
            'i' => {
                self.set_value("pitch", Some(0f64));
            }
            // a5
            'k' => {
                self.set_value("pitch", Some(0f64));
            }
            // a#5
            'o' => {
                self.set_value("pitch", Some(0f64));
            }
            // b5
            'l' => {
                self.set_value("pitch", Some(0f64));
            }
            // c6
            ';' => {
                self.set_value("pitch", Some(0f64));
            }
            // TODO: Need some way to trigger the gate when note is pressed
            // and released. Some random, unneeded character is a temporary
            // (terrible) solution for testing
            ' ' => {
                //let next_value = match *self.out_gate.read().expect("RwLock is poisoned") {
                //// Toggle gate
                //Some(current_val) => {
                //if current_val > 0f64 {
                //Some(0f64)
                //} else {
                //Some(1f64)
                //}
                //},
                //None => None,
                //};
                self.set_value("gate", Some(1f64));
            }
            '*' => {
                self.set_value("gate", Some(0f64));
            }
            _ => {},
        }
    }

}

impl PartialEq for BasicKeyboard {
    fn eq(&self, other: &Self) -> bool {
            self.id == other.id
    }
}

