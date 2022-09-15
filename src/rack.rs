//use std::collections::{HashMap, HashSet};
use hashbrown::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;

use cpal::{SupportedStreamConfig, Host};
use cpal::traits::{HostTrait, DeviceTrait};

use crate::clock::Clock;
use crate::cpal_config::CpalConfig;
use crate::io_module_trait::IoModuleTrait;
use crate::types::{ModuleNotFoundError, SAMPLE_RATE, SampleType};

/// A Rack encompasses a group of conntected modules
pub struct Rack {
    /// A map of IoBlocks, using their IDs as identifier
    pub modules: HashMap<String, Box<dyn IoModuleTrait + Send + Sync>>,

    /// Ordered modules for sequential processing
    //module_chain: HashMap<u64, Vec<String>>,
    module_chain: HashMap<u64, HashSet<String>>,

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
        //let module_chain = Vec::new();
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
    pub fn add_module(&mut self, module: Box<dyn IoModuleTrait + Send + Sync>) {
        self.modules.insert(module.get_id().clone(), module);
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

		let out_port = match out_module.get_out_port_ref(out_port_id) {
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
        if let Some(_) = in_module.get_in_port_ref(in_port_id) {
            in_module.set_in_port(in_port_id, out_port.clone());
        } else {
            // Add previously removed module back before failure
            self.modules.insert(String::from(in_module_id), in_module);
            println!("Module {} does not have a port {}", in_module_id, in_port_id);
            return Err(ModuleNotFoundError);
        }

		match (out_module.get_module_order(), in_module.get_module_order()) {
			(None, None) => {
				out_module.set_module_order(Some(1));
				in_module.set_module_order(Some(2));
			},
			(None, Some(order)) => {
                if order == 1 {
                    out_module.set_module_order(Some(order));
                    in_module.set_module_order(Some(order + 1));
                } else {
                    out_module.set_module_order(Some(order - 1));
                    in_module.set_module_order(Some(order));
                }
			},
			(Some(order), None) => {
				in_module.set_module_order(Some(order + 1));
			},
			(Some(_), Some(order)) => { in_module.set_module_order(Some(order + 1)); },
		}

        // Add entry to module position for this functions order
        self.module_chain.entry(out_module.get_module_order().unwrap())
            // If there's no entry for the key, create a new Vec and return a mutable ref to it
            .or_default()
            // and insert the item onto the Vec
            .insert(out_module.get_id().clone());

        self.module_chain.entry(in_module.get_module_order().unwrap())
            // If there's no entry for the key, create a new Vec and return a mutable ref to it
            .or_default()
            // and insert the item onto the Vec
            .insert(in_module.get_id().clone());

        // Add back previously removed module, with updated ports
        self.modules.insert(String::from(in_module_id), in_module);


		Ok(())
    }


    pub fn disconnect_modules(&mut self) {
        // TODO
    }

    pub fn list_ports(&self) {
        println!("Ports:");
        for (module_id, module) in &self.modules {
            println!("Module - {}:", module_id);

            println!("\tinputs:");
            for id in module.get_in_ports() {
                println!("\t\t{}", id);
            }
            println!("\toutputs:");
            for id in module.get_out_ports() {
                println!("\t\t{}", id);
            }
            println!();
        }
    }

    /// Print the connections between a Rack's items
    pub fn print_connection(&self, module_a: &str, module_b: &str) {
        let module_a = self.modules.get(module_a).unwrap();
        let module_b = self.modules.get(module_b).unwrap();

        println!("{} : {}", module_a.get_id(), module_b.get_id());

        // TODO
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
        let order_max = self.get_order_max().unwrap_or(&0).to_owned();

        // Process modules in order_max
        // FIXME - This has potential to be parallelised as modules of
        // equal order should be able to process at the same time
        for position in 1..=order_max {
            for module in self.module_chain.get_mut(&position).unwrap().iter() {

                let next_module = self.modules.get_mut(module).unwrap();

                next_module.process_inputs();
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


