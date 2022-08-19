use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::io_module_trait::IoModuleTrait;
use crate::io_module::IoModule;
use crate::types::{ModuleNotFoundError, SAMPLE_RATE, SampleType};

/// A Rack encompasses a group of conntected modules
pub struct Rack {
    /// A map of IoBlocks, using their IDs as identifier
    pub modules: HashMap<String, Box<dyn IoModuleTrait>>,

    /// Ordered modules for sequential processing
    //module_chain: HashMap<u64, Vec<String>>,
    module_chain: HashMap<u64, HashSet<String>>,
    //module_chain: Vec<String>,
    time: SampleType,
}

impl Rack {
    /// Create a new, empty Rack
    pub fn new() -> Self {
        let modules = HashMap::new();
        let module_chain = HashMap::new();
        //let module_chain = Vec::new();
        let time = 0.0;

        Self {
            modules,
            module_chain,
            time,
        }
    }

    /// Add a new module to the Rack
    pub fn add_module(&mut self, module: Box<dyn IoModuleTrait>) {
        self.modules.insert(module.get_id().clone(), module);
    }

    pub fn add_audio_output(&mut self, audio_out: IoModule) {
        // TODO

    }

    /// Remove a module from the Rack
    fn remove_module(&mut self, _module: IoModule) {
        // TODO
    }

    /// Conect two modules via the given ports
    pub fn connect_modules(
        &mut self,
        out_module_id: &str,
        out_port_id: &str,
        in_module_id: &str,
        in_port_id: &str,
    ) -> std::result::Result<(), ModuleNotFoundError> {
        // Check for presence of the key and remove the module from the hash map if it exists
        // This is the only way I see to have mutible references to two elements of the HashMap
		let mut in_module = if self.modules.contains_key(in_module_id) {
            self.modules.remove(in_module_id).ok_or(ModuleNotFoundError)
        } else {
            println!("Module {} does not exist", in_module_id);
            Err(ModuleNotFoundError)
        }?;

        let out_module = match self.modules.get_mut(out_module_id) {
            Some(module) => Ok(module),
            None => {
                // Add previously removed module back before failure
                self.modules.insert(String::from(in_module_id), in_module);
                println!("Module {} does not exist", out_module_id);
                return Err(ModuleNotFoundError);
            }
        }?;

		let out_port = out_module.get_out_port_ref(out_port_id).ok_or(ModuleNotFoundError)?;

        // Only connect to existing port, in order to avoid mistakes
        // Note: with this, older connections to the port are automatically disconnected
        //       it's possible that extra handling will be required here
        if let Some(_) = in_module.get_in_port_ref(in_port_id) {
            in_module.set_in_port(in_port_id, out_port.clone());
        } else {
            // Add previously removed module back before failure
            self.modules.insert(String::from(in_module_id), in_module);
            return Err(ModuleNotFoundError);
        }

		match (out_module.get_module_order(), in_module.get_module_order()) {
			(None, None) => {
				out_module.set_module_order(Some(1));
				in_module.set_module_order(Some(2));
			},
			(None, Some(order)) => {
				out_module.set_module_order(Some(order));
				in_module.set_module_order(Some(order + 1));
			},
			(Some(order), None) => {
				in_module.set_module_order(Some(order + 1));
			},
			(Some(_), Some(order)) => { in_module.set_module_order(Some(order + 1)); },
		}

        // Add entry to module position for this functions order
        self.module_chain.entry(out_module.get_module_order().unwrap())
            // If there's no entry for key 3, create a new Vec and return a mutable ref to it
            .or_default()
            // and insert the item onto the Vec
            .insert(out_module.get_id().clone());

        self.module_chain.entry(in_module.get_module_order().unwrap())
            // If there's no entry for key 3, create a new Vec and return a mutable ref to it
            .or_default()
            // and insert the item onto the Vec
            .insert(in_module.get_id().clone());

        // Add back previously removed module, with updated ports
        self.modules.insert(String::from(in_module_id), in_module);


		Ok(())
    }


    pub fn disconnect_modules(
        &mut self,
        out_module_id: &str,
        out_port_id: &str,
        in_module_id: &str,
        in_port_id: &str,
    ) {

        if let Some(out_module) = self.modules.get(out_module_id) {
            if let Some(in_module) = self.modules.get(in_module_id) {
                if let Some(_out_port) = out_module.get_out_port_ref(out_port_id) {
                    if in_module.get_in_ports_ref().contains_key(in_port_id) {
                        let mut in_ports = in_module.get_in_ports_ref().to_owned();
                        in_ports.remove(in_port_id);
                    }
                }
            }
        }
    }

    /// Print the connections between a Rack's items
    pub fn list_ports(&self) {
        println!("Ports:");
        for (module_id, module) in &self.modules {
            println!("Module - {}:", module_id);

            println!("\tinputs:");
            for id in module.get_in_ports_ref().keys() {
                println!("\t\t{}", id);
            }
            println!("\toutputs:");
            for id in module.get_out_ports_ref().keys() {
                println!("\t\t{}", id);
            }
            println!();
        }
    }

    pub fn print_connection(&self, module_a: &str, module_b: &str) {
        let module_a = self.modules.get(module_a).unwrap();
        let module_b = self.modules.get(module_b).unwrap();

        println!("{} : {}", module_a.get_id(), module_b.get_id());
        for (out_port_id, out_value) in module_a.get_out_ports_ref().iter() {
            for (in_port_id, in_value) in module_b.get_in_ports_ref().iter() {
                if Arc::ptr_eq(out_value, in_value) {
                    println!("\t{} <----> {}", out_port_id, in_port_id);
                } else {
                    println!("\t{}        {}", out_port_id, in_port_id);
                }
            }
        }
    }

    pub fn print_module_order(&self) {
        println!("Module order:");
        for (order, modules) in self.module_chain.iter() {
            println!("\tModules in position {}:", order);
            for module in modules {
                println!("\t\t{}", module);
            }
        }
    }

    pub fn process_module_chain(&mut self) {
        let time_delta = 1.0 / (SAMPLE_RATE as SampleType);

        // FIXME: read 'order' in ascending order, e.g. for 1..order_max
        for (order, modules) in self.module_chain.iter() {
            for module in modules {
                println!("processing: {}", module);
                let module = &mut *self.modules.get_mut(module).unwrap();
                module.process_inputs();
            }
        }
        if self.time >= 1.0 {
            //*current_time -= 1.0;
            self.time = 0.0;
        } else {
            self.time += time_delta;
        }
    }

}


