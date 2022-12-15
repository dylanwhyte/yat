use std::sync::{Arc, RwLock};

use crate::types::{IoPort, PortResult, PortNotFoundError};
use crate::modules::io_module::IoModule;

/// A module which multiplies its input signals and
/// outputs the result
pub struct Multiplier {
    /// A unique string used for identifying the module
    id: String,

    /// Order of the module in the chain, where 0 (zero) means skipped
    order: Option<u64>,

    input_ports: Vec<String>,

    output_ports: Vec<String>,

    in_a: IoPort,

    in_b: IoPort,

    out_result: IoPort,
}

impl Multiplier {
    /// Create a new, unordered IoModule
    pub fn new(id: String) -> Self {
        let order = None;
        let input_ports = vec!["in_a".to_string(), "in_b".to_string()];
        let output_ports = vec!["result".to_string()];

        let in_a = Arc::new(RwLock::new(None));
        let in_b = Arc::new(RwLock::new(None));
        let out_result = Arc::new(RwLock::new(None));

        Self {
            id,
            order,
            input_ports,
            output_ports,
            in_a,
            in_b,
            out_result,
        }
    }

}

impl PartialEq for Multiplier {
    fn eq(&self, other: &Self) -> bool {
            self.id == other.id
    }
}

impl IoModule for Multiplier {
    /// Read inputs and populate outputs
    fn process_inputs(&mut self) {
        let a = self.in_a.read().expect("RwLock is poisoned").unwrap_or(0f64);
        let b = self.in_b.read().expect("RwLock is poisoned").unwrap_or(0f64);

        let result = a * b;

        let mut value = self.out_result.write().expect("RwLock is poisoned");
        *value = Some(result);
    }

    /// Return a module's ID
    fn get_id(&self) -> &String {
        &self.id
    }

    fn get_in_ports(&self) -> &Vec<String> {
        &self.input_ports
    }

    fn get_out_ports(&self) -> &Vec<String> {
        &self.output_ports
    }

    /// Return a reference to one of the module's input ports
    fn get_in_port_ref(&self, port_id: &str) -> Option<IoPort> {
        match port_id {
            "a" => Some(self.in_a.clone()),
            "b" => Some(self.in_b.clone()),
            _ => None,
        }
    }

    fn get_out_port_ref(&self, port_id: &str) -> Option<IoPort> {
        match port_id {
            "result" => Some(self.out_result.clone()),
            _ => None,
        }
    }

    /// Set the value of a module's input port
    fn set_in_port(&mut self, port_id: &str, out_port: IoPort) -> PortResult<String> {
        match port_id {
            "a" => self.in_a = out_port,
            "b" => self.in_b = out_port,
            _ => { return Err(PortNotFoundError) },
        }

        Ok(format!("{}: Set port {}\n", self.get_id(), port_id))
    }

    fn get_module_order(&self) -> Option<u64> {
        self.order
    }

    fn set_module_order(&mut self, new_order: Option<u64>) {
        self.order = new_order;
    }
}

