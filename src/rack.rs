use hashbrown::{HashMap, HashSet};
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::AtomicBool;

use cpal::{SupportedStreamConfig, Host};
use cpal::traits::{HostTrait, DeviceTrait};

use crate::clock::Clock;
use crate::cpal_config::CpalConfig;
use crate::io_module::IoModule;
use crate::types::{ModuleNotFoundError, SAMPLE_RATE, SampleType};
use crate::oscillator::Oscillator;

/// A Rack encompasses a group of conntected modules
pub struct Rack {
    /// A map of IoBlocks, using their IDs as identifier
    modules: HashMap<String, Arc<Mutex<dyn IoModule + Send + Sync>>>,

    /// Ordered modules for sequential processing
    module_chain: HashMap<u64, Vec<Arc<Mutex<dyn IoModule + Send + Sync>>>>,

    //module_chain: Vec<String>,
    pub clock: Clock,

    pub running: AtomicBool,

    // cpal host, device and config
    cpal_config: Arc<RwLock<CpalConfig>>,
}

impl Rack {
    /// Create a new, empty Rack
    pub fn new() -> Self {
        let modules = HashMap::new();
        let module_chain = HashMap::new();
        let clock = Clock::new();
        let running = AtomicBool::new(false);

        let cpal_config = Arc::new(RwLock::new(CpalConfig::new()));


        Self {
            modules,
            module_chain,
            clock,
            running,
            cpal_config,
        }
    }

    /// Return the cpal config
    pub fn get_cpal_config(&self) -> Arc<RwLock<CpalConfig>> {
        self.cpal_config.clone()
    }

    /// Add a new module to the Rack
    pub fn add_module(&mut self, module: Arc<Mutex<dyn IoModule + Send + Sync>>) {
        let module_id = { module.lock().unwrap().get_id().clone() };
        self.modules.insert(module_id, module);
    }

    pub fn add_module_type(&mut self, module_type: &str, module_id: &str) {
        let module = match module_type {
            "osc" => {
                Oscillator::new(module_id.into(), self.clock.time.clone())
            },
            _ => { println!("module type '{}' does not exist", module_type); return; },
        };

        self.modules.insert(module_id.into(), Arc::new(Mutex::new(module)));
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

        let in_module = if self.modules.contains_key(in_module_id) {
            self.modules.remove(in_module_id).ok_or(ModuleNotFoundError)
        } else {
            println!("Module {} does not exist", in_module_id);
            Err(ModuleNotFoundError)
        }?;


        let out_module = match self.modules.get_mut(out_module_id) {
            Some(module) => Ok(module),
            None => {
                // Add previously removed module back before failure
                println!("Reinserting module after failure");
                self.modules.insert(String::from(in_module_id), in_module);
                println!("Module {} does not exist", out_module_id);
                return Err(ModuleNotFoundError);
            }
        }?;

        let port_id = { out_module.lock().unwrap().get_out_port_ref(out_port_id) };
        let out_port = match port_id {
            Some(out_port) => out_port,
            None => {
                self.modules.insert(String::from(in_module_id), in_module);
                println!("Module {} does not have a port {}", out_module_id, out_port_id);
                return Err(ModuleNotFoundError);
            },
        };

        // Only connect to existing port, in order to avoid mistakes
        // Note: with this, older connections to the port are automatically disconnected
        //       it's possible that extra handling will be required here
        let port_id = { in_module.lock().unwrap().get_in_port_ref(in_port_id) };
        if let Some(_) = port_id {
            in_module.lock().unwrap().set_in_port(in_port_id, out_port.clone());
        } else {
            // Add previously removed module back before failure
            self.modules.insert(String::from(in_module_id.clone()), in_module);
            println!("Module {} does not have a port {}", in_module_id, in_port_id);
            return Err(ModuleNotFoundError);
        }

        let out_module_order = { out_module.lock().unwrap().get_module_order() };
        let in_module_order = { in_module.lock().unwrap().get_module_order() };

        match (out_module_order, in_module_order) {
            (None, None) => {
                out_module.lock().unwrap().set_module_order(Some(1));
                in_module.lock().unwrap().set_module_order(Some(2));
            },
            (None, Some(order)) => {
                // TODO: is there a faster way to remove an item at an unknown location
                self.module_chain.get_mut(&order).unwrap()
                    .retain(|module| {
                        !Arc::ptr_eq(module, &in_module)
                    });
                if order == 1 {
                    out_module.lock().unwrap().set_module_order(Some(order));
                    in_module.lock().unwrap().set_module_order(Some(order + 1));
                } else {
                    out_module.lock().unwrap().set_module_order(Some(order - 1));
                    in_module.lock().unwrap().set_module_order(Some(order));
                }
            },
            (Some(order), None) => {
                self.module_chain.get_mut(&order).unwrap()
                    .retain(|module| {
                        !Arc::ptr_eq(module, &out_module)
                    });
                in_module.lock().unwrap().set_module_order(Some(order + 1));
            },
            (Some(_), Some(order)) => {
                self.module_chain.get_mut(&out_module_order.unwrap()).unwrap()
                    .retain(|module| {
                        !Arc::ptr_eq(module, &out_module)
                    });
                self.module_chain.get_mut(&order).unwrap()
                    .retain(|module| {
                        !Arc::ptr_eq(module, &in_module)
                    });
                in_module.lock().unwrap().set_module_order(Some(order + 1));
            },
        }

        // Add entry to module position for this functions order
        self.module_chain.entry(out_module.lock().unwrap().get_module_order().unwrap())
            // If there's no entry for the key, create a new Vec and return a mutable ref to it
            .or_default()
            // and insert the item onto the Vec
            //.insert(out_module.get_id().clone());
            .push(out_module.clone());

        self.module_chain.entry(in_module.lock().unwrap().get_module_order().unwrap())
            // If there's no entry for the key, create a new Vec and return a mutable ref to it
            .or_default()
            // and insert the item onto the Vec
            .push(in_module.clone());

        // Add back previously removed module, with updated ports
        self.modules.insert(String::from(in_module_id), in_module);

        Ok(())
    }


    pub fn disconnect_modules(&mut self) {
        // TODO
    }

	pub fn print_ports(&self, module_id: Option<&str>) -> String {
		let mut output = String::from("Ports: \n");
		if let Some(module_id) = module_id {
			//if self.modules.contains_key(module_id) {
			if let Some(module) = self.modules.get(module_id) {
				output.push_str("Module - ");
				output.push_str(module_id);
				output.push_str(":\n");

				output.push_str("    inputs:\n");
				for id in module.lock().unwrap().get_in_ports() {
					output.push_str("        ");
					output.push_str(id);
					output.push_str("\n");
				}
				output.push_str("    outputs:\n");
				for id in module.lock().unwrap().get_out_ports() {
					output.push_str("        ");
					output.push_str(id);
					output.push_str("\n");
				}
			}
		} else {
			for (module_id, module) in &self.modules {
				output.push_str("Module - ");
				output.push_str(module_id);
				output.push_str(":\n");

				output.push_str("    inputs:\n");
				for id in module.lock().unwrap().get_in_ports() {
					output.push_str("        ");
					output.push_str(id);
					output.push_str("\n");
				}
				output.push_str("    outputs:\n");
				for id in module.lock().unwrap().get_out_ports() {
					output.push_str("        ");
					output.push_str(id);
					output.push_str("\n");
				}
			}
		}

		output.push_str("\n");
		output
		}

    /// Print the connections between a Rack's items
    pub fn print_connection(&self, module_a: &str, module_b: &str) {
        //let module_a = self.modules.get(module_a).unwrap();
        //let module_b = self.modules.get(module_b).unwrap();

        //println!("{} : {}", module_a.lock().unwrap().get_id(), module_b.lock().unwrap().get_id());

        // TODO
    }

    pub fn print_module_order(&self) -> String {
		let mut output = String::from("Module order:\n");
        for (order, modules) in self.module_chain.iter() {
            output.push_str("    Modules in position ");
            output.push_str(&order.to_string());
            output.push_str(":\n");
            for module in modules {
                output.push_str("        ");
				output.push_str(module.lock().unwrap().get_id());
                output.push_str("\n");
            }
        }

		output.push_str("\n");
		output
    }

    pub fn print_modules(&self) -> String {
		let mut output = String::from("Modules:\n");
        for module in self.modules.keys() {
			output.push_str("    ");
			output.push_str(module);
			output.push_str("\n");
        }

		output.push_str("\n");
		output
    }

    pub fn process_module_chain(&mut self) {
        let order_max = self.get_order_max().unwrap_or(&0).to_owned();

        // Process modules in order
        // FIXME - This has potential to be parallelised as modules of
        // equal order should be able to process at the same time
        for position in 1..=order_max {
            for module in self.module_chain.get_mut(&position).unwrap() {
                module.lock().unwrap().process_inputs();
            }
        }

        // After each module has been processed update the time for the next round of processing
        self.clock.increment();
    }

    // Returns the highest value order in the module_chain hashmap
    fn get_order_max(&self) -> Option<&u64> {
        self.module_chain.keys().to_owned().reduce(
            |accum, item| {
                if accum >= item { accum } else { item }
            }
        )
    }

    pub fn run(&mut self) {
        *self.running.get_mut() = true;
    }

    pub fn stop(&mut self) {
        *self.running.get_mut() = false;
    }

}


