use std::sync::{Weak, RwLock};

use crate::controls::control::Control;
use crate::types::SampleType;
use crate::out_port::OutPort;


/// An control IoModule
pub struct BasicKeyboard {
    /// A unique string used for identifying the module
    id: String,

    /// A gate signal to communicate when a note is activated
    /// and deactivated
    out_gate: OutPort,

    /// The pitch representation of the note that is pressed
    out_pitch: OutPort,

    /// Output velocity.
    out_velocity: OutPort,
}

impl BasicKeyboard {
    /// Create a new BasicKeyboard
    pub fn new(id: String) -> Self {
        let out_gate = OutPort::new("gate".into());
        let out_pitch = OutPort::new("pitch".into());
        let out_velocity = OutPort::new("velocity".into());

        Self {
            id,
            out_gate,
            out_pitch,
            out_velocity,
        }
    }
}

impl Control for BasicKeyboard {
    /// Get a reference to the control's output port
    fn get_port_reference(&self, port: &str)
        -> Option<Weak<RwLock<Option<SampleType>>>> {
        match port {
            "gate" => Some(self.out_gate.get_ref()),
            "pitch" => Some(self.out_pitch.get_ref()),
            "velocity" => Some(self.out_velocity.get_ref()),
            _ => None,
        }
    }

    /// Set the controls output value
    fn set_value(&self, port: &str, new_value: SampleType) {
        match port {
            "gate" => self.out_gate.set_value(new_value),
            "pitch" => self.out_pitch.set_value(new_value),
            "velocity" => self.out_velocity.set_value(new_value),
            _ => (),
        }
    }

    /// Receive and handle a control keys.
    /// For a button, the spacebar toggles the button on and off
    fn recv_control_key(&self, key: char) {
        match key {
            // a4
            'a' => {
                self.set_value("pitch", 440f64);
            }
            // a#4
            'w' => {
                self.set_value("pitch", 466.16f64);
            }
            // b4
            's' => {
                self.set_value("pitch", 493.88f64);
            }
            // c5
            'd' => {
                self.set_value("pitch", 523.25f64);
            }
            // c#5
            'r' => {
                self.set_value("pitch", 554.37f64);
            }
            // d5
            'f' => {
                self.set_value("pitch", 587.33f64);
            }
            // d#5
            't' => {
                self.set_value("pitch", 622.25f64);
            }
            // e5
            'g' => {
                self.set_value("pitch", 659.26f64);
            }
            // f5
            'h' => {
                self.set_value("pitch", 698.46f64);
            }
            // f#5
            'u' => {
                self.set_value("pitch", 0f64);
            }
            // g5
            'j' => {
                self.set_value("pitch", 0f64);
            }
            // g#5
            'i' => {
                self.set_value("pitch", 0f64);
            }
            // a5
            'k' => {
                self.set_value("pitch", 0f64);
            }
            // a#5
            'o' => {
                self.set_value("pitch", 0f64);
            }
            // b5
            'l' => {
                self.set_value("pitch", 0f64);
            }
            // c6
            ';' => {
                self.set_value("pitch", 0f64);
            }
            // TODO: Need some way to trigger the gate when note is pressed
            // and released. Some random, unneeded character is a temporary
            // (terrible) solution for testing
            ' ' => {
                self.set_value("gate", 1f64);
            }
            '*' => {
                self.set_value("gate", 0f64);
            }
            _ => {}
        }
    }
}

impl PartialEq for BasicKeyboard {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
