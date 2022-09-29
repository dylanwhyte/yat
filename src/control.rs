use std::sync::{Arc, RwLock};

use crate::types::{IoPort, SampleType};

/// An control IoModule
pub struct Control {
    /// A unique string used for identifying the module
    id: String,

    /// The control's output value
    out_value: IoPort,
}

impl Control {
    /// Create a new, unordered IoModule
    pub fn new(id: String) -> Self {
        let out_value = Arc::new(RwLock::new(None));

        Self {
            id,
            out_value,
        }
    }

    pub fn get_port_reference(&self) -> IoPort {
        self.out_value.clone()
    }

    pub fn set_value(&self, new_value: Option<SampleType>) {
        if let Ok(mut value) = self.out_value.write() {
            *value = new_value;
        }
    }

}

impl PartialEq for Control {
    fn eq(&self, other: &Self) -> bool {
            self.id == other.id
    }
}

