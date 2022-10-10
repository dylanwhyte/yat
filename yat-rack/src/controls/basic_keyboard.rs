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
        let out_gate = Arc::new(RwLock::new(Some(0f32)));
        let out_pitch = Arc::new(RwLock::new(Some(0f32)));

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
                if let Ok(mut value) = self.out_gate.write() {
                    *value = new_value;
                }
            },
            "pitch" => {
                if let Ok(mut value) = self.out_pitch.write() {
                    *value = new_value;
                }
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
                self.set_value("pitch", Some(440f32));
            },
            // a#4
            'w' => {
                self.set_value("pitch", Some(466.16f32));
            },
            // b4
            's' => {
                self.set_value("pitch", Some(493.88f32));
            },
            // c5
            'd' => {
                self.set_value("pitch", Some(523.25f32));
            },
            // c#5
            'r' => {
                self.set_value("pitch", Some(554.37f32));
            },
            // d5
            'f' => {
                self.set_value("pitch", Some(587.33f32));
            },
            // d#5
            't' => {
                self.set_value("pitch", Some(622.25f32));
            },
            // e5
            'g' => {
                self.set_value("pitch", Some(659.26f32));
            },
            // f5
            'h' => {
                self.set_value("pitch", Some(698.46f32));
            },
            // f#5
            'u' => {
                self.set_value("pitch", Some(0f32));
            },
            // g5
            'j' => {
                self.set_value("pitch", Some(0f32));
            },
            // g#5
            'i' => {
                self.set_value("pitch", Some(0f32));
            },
            // a5
            'k' => {
                self.set_value("pitch", Some(0f32));
            },
            // a#5
            'o' => {
                self.set_value("pitch", Some(0f32));
            },
            // b5
            'l' => {
                self.set_value("pitch", Some(0f32));
            },
            // c6
            ';' => {
                self.set_value("pitch", Some(0f32));
            },
            // TODO: Need some way to trigger the gate when note is pressed
            // and released. Some random, unneeded character is a temporary
            // (terrible) solution for testing
            ' ' => {
                let next_value = match *self.out_gate.read().unwrap() {
                    // Toggle gate
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
            _ => {},
        }
    }

}

impl PartialEq for BasicKeyboard {
    fn eq(&self, other: &Self) -> bool {
            self.id == other.id
    }
}

