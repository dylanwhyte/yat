#![allow(dead_code)]
//use std::collections::{HashMap, HashSet};
use hashbrown::{HashMap, HashSet};
use std::fmt;

use crate::types::{IoPort, SampleType};

pub trait IoModuleTrait {
    /// Calculate the module's outputs based on inputs
    fn process_inputs(&mut self);

    /// Get the module's unique ID
    fn get_id(&self) -> &String;

    /// Returns the module's input ports
    fn get_in_ports(&self) -> &Vec<String>;

    /// Returns the module's output ports
    fn get_out_ports(&self) -> &Vec<String>;

    /// Returns a reference to a single input port
    fn get_in_port_ref(&self, port_id: &str) -> Option<IoPort>;

    /// Returns a reference to a single output port
    fn get_out_port_ref(&self, port_id: &str) -> Option<IoPort>;

    /// Set the value of a module's input port
    fn set_in_port(&mut self, port_id: &str, out_port: IoPort);

    /// Get a modules processing order
    fn get_module_order(&self) -> Option<u64>;

    /// Set the modules processing order
    fn set_module_order(&mut self, new_order: Option<u64>);

}


