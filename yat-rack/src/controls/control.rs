use crate::types::{IoPort, SampleType};

/// A trait for implementng controls.
/// In the context of a Rack, controls are a special type of module which are not ordered, as they
/// are controlled concurrently and don't take an input from other modules.
pub trait Control {
    /// Get a reference to the control's output port
    fn get_port_reference(&self, port: &str) -> Option<IoPort>;

    /// Set the controls output value
    fn set_value(&self, port: &str, new_value: Option<SampleType>);

    /// Receive and handle a control key. This allows the control to listen for commands and update
    /// it's output accordingly (somewhat akin to a module's processing function)
    fn recv_control_key(&self, key: char);
}
