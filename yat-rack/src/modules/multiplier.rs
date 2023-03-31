use std::sync::{RwLock, Weak};

use crate::in_port::InPort;
use crate::modules::io_module::IoModule;
use crate::out_port::OutPort;
use crate::types::{PortNotFoundError, PortResult, SampleType};

/// A module which multiplies its input signals and
/// outputs the result
pub struct Multiplier {
    /// A unique string used for identifying the module
    id: String,

    /// Order of the module in the chain, where 0 (zero) means skipped
    order: Option<u64>,

    input_ports: Vec<String>,

    output_ports: Vec<String>,

    in_a: InPort,

    in_b: InPort,

    out_mult: OutPort,
}

impl Multiplier {
    /// Create a new, unordered IoModule
    pub fn new(id: String) -> Self {
        let order = None;
        let input_ports = vec!["in_a".to_string(), "in_b".to_string()];
        let output_ports = vec!["result".to_string()];

        let in_a = InPort::new("a".into(), SampleType::MIN, SampleType::MAX, 0.0);
        let in_b = InPort::new("b".into(), SampleType::MIN, SampleType::MAX, 0.0);
        let out_mult = OutPort::new("mult".into());

        Self {
            id,
            order,
            input_ports,
            output_ports,
            in_a,
            in_b,
            out_mult,
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
        let a = self.in_a.get_value();
        let b = self.in_b.get_value();

        let mult = a * b;

        self.out_mult.set_value(mult);
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
    fn has_port_with_id(&self, port_id: &str) -> bool {
        match port_id {
            "a" | "b" => true,
            _ => false,
        }
    }

    fn get_out_port_ref(&self, port_id: &str) -> Option<&OutPort> {
        match port_id {
            "result" => Some(&self.out_mult),
            _ => None,
        }
    }

    /// Set the value of a module's input port
    fn set_in_port(&mut self, port_id: &str, out_port_ref: Weak<RwLock<Option<SampleType>>>) -> PortResult<String> {
        match port_id {
            "a" => self.in_a.set_value(out_port_ref),
            "b" => self.in_b.set_value(out_port_ref),
            _ => return Err(PortNotFoundError),
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
