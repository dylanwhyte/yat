use hashbrown::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};

use crate::clock::Clock;
use crate::controls::basic_keyboard::BasicKeyboard;
use crate::controls::button::Button;
use crate::controls::control::Control;
use crate::controls::control_knob::ControlKnob;
use crate::modules::adsr::Adsr;
use crate::modules::io_module::IoModule;
use crate::modules::oscillator::Oscillator;
use crate::types::{
    ConflictingModuleIdError, ModuleNotFoundError, ModuleResult, PortNotFoundError, SampleType,
};

/// A Rack encompasses a group of conntected modules
pub struct Rack {
    /// A map of IoBlocks, using their IDs as identifier
    modules: HashMap<String, Arc<Mutex<dyn IoModule + Send + Sync>>>,

    /// Controls: these don't require an order to be processed
    controls: HashMap<String, Arc<Mutex<dyn Control + Send + Sync>>>,

    /// The control which currently holds the focus
    focussed_control: Option<Arc<Mutex<dyn Control + Send + Sync>>>,

    /// Ordered modules for sequential processing
    module_chain: HashMap<u64, Vec<Arc<Mutex<dyn IoModule + Send + Sync>>>>,

    /// The Rack's clock keeps track of timing. This is passed to modules
    /// whose output rely on time
    pub clock: Arc<RwLock<Clock>>,

    /// Determines the rack is in a running/processing or stopped state
    pub running: AtomicBool,
}

impl Rack {
    /// Create a new, empty Rack
    pub fn new() -> Self {
        let modules = HashMap::new();
        let controls = HashMap::new();
        let focussed_control = None;
        let module_chain = HashMap::new();
        let clock = Arc::new(RwLock::new(Clock::new()));
        let running = AtomicBool::new(true);

        Self {
            modules,
            controls,
            focussed_control,
            module_chain,
            clock,
            running,
        }
    }

    /// Add a new module to the Rack
    pub fn add_module(&mut self, module: Arc<Mutex<dyn IoModule + Send + Sync>>) {
        let module_id = module.lock().expect("Mutex poisoned").get_id().clone();
        self.modules.insert(module_id, module);
    }

    pub fn add_module_type(
        &mut self,
        module_type: &str,
        module_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Dissallow adding modules/controls with identical IDs, so that there can be no
        // ambiguity when connecting
        if self.modules.contains_key(module_id) || self.controls.contains_key(module_id) {
            return Err(Box::new(ConflictingModuleIdError));
        }

        match module_type {
            // Controls
            "control" => {
                let control_knob = Arc::new(Mutex::new(ControlKnob::new(module_id.into())));
                self.controls.insert(module_id.into(), control_knob);
            }
            "button" => {
                let button = Arc::new(Mutex::new(Button::new(module_id.into())));
                self.controls.insert(module_id.into(), button);
            }
            "keyboard" => {
                let keyboard = Arc::new(Mutex::new(BasicKeyboard::new(module_id.into())));
                self.controls.insert(module_id.into(), keyboard);
            }
            // Modules
            "osc" => {
                let oscillator = Arc::new(Mutex::new(Oscillator::new(
                    module_id.into(),
                    self.clock.clone(),
                )));
                self.modules.insert(module_id.into(), oscillator);
            }
            "adsr" => {
                let adsr = Arc::new(Mutex::new(Adsr::new(module_id.into(), self.clock.clone())));
                self.modules.insert(module_id.into(), adsr);
            }
            _ => return Err(Box::new(ModuleNotFoundError)),
        }

        Ok(format!("Add {} with id {}", module_type, module_id))
    }

    /// Conect two modules via the given ports
    pub fn connect_modules(
        &mut self,
        out_module_id: &str,
        out_port_id: &str,
        in_module_id: &str,
        in_port_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Check for presence of the key and remove the module from the hash map if it exists
        // This is the only way I see to have mutible references to two elements of the HashMap

        if self.controls.contains_key(out_module_id) {
            match self.connect_ctrl(out_module_id, out_port_id, in_module_id, in_port_id) {
                Ok(res) => return Ok(res),
                Err(e) => return Err(e),
            }
        }

        let in_module = if self.modules.contains_key(in_module_id) {
            self.modules.remove(in_module_id).ok_or(ModuleNotFoundError)
        } else {
            println!("Module {} does not exist", in_module_id);
            Err(ModuleNotFoundError)
        }?;

        let out_module = match self.modules.get_mut(out_module_id) {
            Some(module) => Ok::<_, ModuleNotFoundError>(module),
            None => {
                // Add previously removed module back before failure
                println!("Reinserting module after failure");
                self.modules.insert(String::from(in_module_id), in_module);
                println!("Module {} does not exist", out_module_id);
                return Err(Box::new(ModuleNotFoundError));
            }
        }?;

        let port_id = {
            out_module
                .lock()
                .expect("Mutex lock is poisoned")
                .get_out_port_ref(out_port_id)
        };

        let out_port = match port_id {
            Some(out_port) => out_port,
            None => {
                self.modules.insert(String::from(in_module_id), in_module);
                println!(
                    "Module {} does not have a port {}",
                    out_module_id, out_port_id
                );
                return Err(Box::new(ModuleNotFoundError));
            }
        };

        // Only connect to existing port, in order to avoid mistakes
        // Note: with this, older connections to the port are automatically disconnected
        //       it's possible that extra handling will be required here
        let port_id = {
            in_module
                .lock()
                .expect("Mutex lock is poisoned")
                .get_in_port_ref(in_port_id)
        };

        if port_id.is_some() {
            in_module
                .lock()
                .expect("Mutex lock is poisoned")
                .set_in_port(in_port_id, out_port)?;
        } else {
            // Add previously removed module back before failure
            self.modules.insert(String::from(in_module_id), in_module);
            return Err(Box::new(ModuleNotFoundError));
        }

        let out_module_order = {
            out_module
                .lock()
                .expect("Mutex lock is poisoned")
                .get_module_order()
        };
        let in_module_order = {
            in_module
                .lock()
                .expect("Mutex lock is poisoned")
                .get_module_order()
        };

        match (out_module_order, in_module_order) {
            (None, None) => {
                out_module
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .set_module_order(Some(1));
                in_module
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .set_module_order(Some(2));
            }
            (None, Some(order)) => {
                self.module_chain
                    .get_mut(&order)
                    .unwrap()
                    .retain(|module| !Arc::ptr_eq(module, &in_module));
                if order == 1 {
                    out_module
                        .lock()
                        .expect("Mutex lock is poisoned")
                        .set_module_order(Some(order));
                    in_module
                        .lock()
                        .expect("Mutex lock is poisoned")
                        .set_module_order(Some(order + 1));
                } else {
                    out_module
                        .lock()
                        .expect("Mutex lock is poisoned")
                        .set_module_order(Some(order - 1));
                    in_module
                        .lock()
                        .expect("Mutex lock is poisoned")
                        .set_module_order(Some(order));
                }
            }
            (Some(order), None) => {
                self.module_chain
                    .get_mut(&order)
                    .unwrap()
                    .retain(|module| !Arc::ptr_eq(module, out_module));
                in_module
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .set_module_order(Some(order + 1));
            }
            (Some(_), Some(order)) => {
                self.module_chain
                    .get_mut(&out_module_order.unwrap())
                    .unwrap()
                    .retain(|module| !Arc::ptr_eq(module, out_module));
                self.module_chain
                    .get_mut(&order)
                    .unwrap()
                    .retain(|module| !Arc::ptr_eq(module, &in_module));
                in_module
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .set_module_order(Some(order + 1));
            }
        }

        // Add entry to module position for this functions order
        self.module_chain
            .entry(
                out_module
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .get_module_order()
                    .unwrap(),
            )
            // If there's no entry for the key, create a new Vec and return a mutable ref to it
            .or_default()
            // and insert the item onto the Vec
            //.insert(out_module.get_id().clone());
            .push(out_module.clone());

        self.module_chain
            .entry(
                in_module
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .get_module_order()
                    .unwrap(),
            )
            // If there's no entry for the key, create a new Vec and return a mutable ref to it
            .or_default()
            // and insert the item onto the Vec
            .push(in_module.clone());

        // Add back previously removed module, with updated ports
        self.modules.insert(String::from(in_module_id), in_module);

        Ok(format!(
            "connected module {} -> {} to {} -> {}",
            out_module_id, out_port_id, in_module_id, in_port_id
        ))
    }

    pub fn disconnect_module(
        &mut self,
        module_id: &str,
        port_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let module = match self.modules.get(module_id) {
            Some(module) => module,
            None => return Err(Box::new(ModuleNotFoundError)),
        };

        module
            .lock()
            .expect("Mutex lock is poisoned")
            .set_in_port(port_id, Arc::new(RwLock::new(None)))?;

        Ok(format!(
            "disconnected {} from module {}",
            port_id, module_id
        ))
    }

    pub fn connect_ctrl(
        &mut self,
        ctrl_id: &str,
        ctrl_port_id: &str,
        in_module_id: &str,
        in_port_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let control = match self.controls.get(ctrl_id) {
            Some(control) => control,
            None => return Err(Box::new(ModuleNotFoundError)),
        };

        let ctrl_port = match control
            .lock()
            .expect("Mutex lock is poisoned")
            .get_port_reference(ctrl_port_id)
        {
            Some(port) => port,
            None => return Err(Box::new(PortNotFoundError)),
        };

        let in_module = self.modules.get(in_module_id);

        let message = match in_module {
            Some(module) => module
                .lock()
                .expect("Mutex lock is poisoned")
                .set_in_port(in_port_id, ctrl_port)?,
            None => return Err(Box::new(ModuleNotFoundError)),
        };

        Ok(format!(
            "connected control {} -> {} to module {} -> {}. {}",
            ctrl_id, ctrl_port_id, in_module_id, in_port_id, message
        ))
    }

    pub fn set_focus_control(&mut self, ctrl_id: &str) -> ModuleResult<String> {
        let control = self.controls.get(ctrl_id);
        match control {
            Some(ctrl) => self.focussed_control = Some(ctrl.clone()),
            None => return Err(ModuleNotFoundError),
        }

        Ok(format!("{} focussed", ctrl_id))
    }

    pub fn send_control_key(&self, key: char) {
        if let Some(control) = &self.focussed_control {
            control
                .lock()
                .expect("Mutex lock is poisoned")
                .recv_control_key(key);
        }
    }

    pub fn set_ctrl_value(
        &mut self,
        ctrl_id: &str,
        port_id: &str,
        value: Option<SampleType>,
    ) -> ModuleResult<String> {
        match self.controls.get(ctrl_id) {
            Some(ctrl) => ctrl
                .lock()
                .expect("Mutex lock is poisoned")
                .set_value(port_id, value),
            None => return Err(ModuleNotFoundError),
        }

        Ok(format!("Updated control {}", ctrl_id))
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
                for id in module
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .get_in_ports()
                {
                    output.push_str("        ");
                    output.push_str(id);
                    output.push('\n');
                }
                output.push_str("    outputs:\n");
                for id in module
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .get_out_ports()
                {
                    output.push_str("        ");
                    output.push_str(id);
                    output.push('\n');
                }
            }
        } else {
            for (module_id, module) in &self.modules {
                output.push_str("Module - ");
                output.push_str(module_id);
                output.push_str(":\n");

                output.push_str("    inputs:\n");
                for id in module
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .get_in_ports()
                {
                    output.push_str("        ");
                    output.push_str(id);
                    output.push('\n');
                }
                output.push_str("    outputs:\n");
                for id in module
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .get_out_ports()
                {
                    output.push_str("        ");
                    output.push_str(id);
                    output.push('\n');
                }
            }
        }

        output.push('\n');
        output
    }

    /// Print the connections between a Rack's items
    pub fn print_connection(&self, _module_a: &str, _module_b: &str) {
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
                output.push_str(module.lock().expect("Mutex lock is poisoned").get_id());
                output.push('\n');
            }
        }

        output.push('\n');
        output
    }

    pub fn print_modules(&self) -> String {
        let mut output = String::from("Modules:\n");
        for module in self.modules.keys() {
            output.push_str("    ");
            output.push_str(module);
            output.push('\n');
        }

        output.push('\n');

        output.push_str("Controls:\n");
        for control in self.controls.keys() {
            output.push_str("    ");
            output.push_str(control);
            output.push('\n');
        }

        output.push('\n');

        output
    }

    pub fn process_module_chain(&mut self) {
        let order_max = self.get_order_max().unwrap_or(&0).to_owned();

        // Process modules in order
        // FIXME - This has potential to be parallelised as modules of
        // equal order should be able to process at the same time
        for position in 1..=order_max {
            for module in self.module_chain.get(&position).unwrap() {
                module
                    .lock()
                    .expect("Mutex lock is poisoned")
                    .process_inputs();
            }
        }

        // After each module has been processed update the time for the next round of processing
        self.clock.write().expect("RwLock is poisoned").increment();
    }

    // Returns the highest value order in the module_chain hashmap
    fn get_order_max(&self) -> Option<&u64> {
        self.module_chain.keys().to_owned().reduce(
            |accum, item| {
                if accum >= item {
                    accum
                } else {
                    item
                }
            },
        )
    }

    pub fn run(&mut self) {
        *self.running.get_mut() = true;
    }

    pub fn stop(&mut self) {
        *self.running.get_mut() = false;
    }
}

impl Default for Rack {
    fn default() -> Self {
        Self::new()
    }
}
