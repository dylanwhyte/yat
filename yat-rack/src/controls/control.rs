use std::sync::{RwLock, Weak};

use crate::types::SampleType;

/// A trait for implementng controls.
/// In the context of a Rack, controls are a special type of module which are not ordered, as they
/// are controlled concurrently and don't take an input from other modules.
pub trait Control {
    /// Get a reference to the control's output port
    fn get_port_reference(&self, port: &str)
        -> Option<Weak<RwLock<Option<SampleType>>>>;

    /// Set the controls output value
    fn set_value(&self, port: &str, new_value: SampleType);

    /// Receive and handle a control key. This allows the control to listen for commands and update
    /// it's output accordingly (somewhat akin to a module's processing function)
    fn recv_control_key(&self, key: char);

    fn recv_midi(&self, _message: &[u8]) {
        unimplemented!();
    }
}
