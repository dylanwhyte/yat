#![allow(dead_code)]
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::types::{IoPort, SampleType};

pub trait IoModuleTrait {
    /// Get the module's unique ID
    fn get_id(&self) -> &String;

    /// Add an input or output port to the module
    fn create_port(&mut self, port_type: &str, port_name: &str);

    /// Returns a reference to the modules input ports
    fn get_in_ports_ref(&self) -> &HashMap<String, IoPort>;

    /// Returns a reference to a single input port
    fn get_in_port_ref(&self, port_id: &str) -> Option<IoPort>;

    /// Set the value of a module's input port
    fn set_in_port(&mut self, port_id: &str, out_port: IoPort);

    /// Returns the value at an input port
    fn read_in_port_value(&self, in_port_label: &str) -> Option<SampleType>;

    /// Returns a reference to the modules output ports
    fn get_out_ports_ref(&self) -> &HashMap<String, IoPort>;

    /// Returns a reference to a single output port
    fn get_out_port_ref(&self, port_id: &str) -> Option<IoPort>;

    fn write_out_port_value(&self, out_port_label: &str, new_value: Option<SampleType>);

    /// Get a modules processing order
    fn get_module_order(&self) -> Option<u64>;

    /// Set the modules processing order
    fn set_module_order(&mut self, new_order: Option<u64>);
}


