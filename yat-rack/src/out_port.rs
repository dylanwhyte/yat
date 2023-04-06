use std::sync::{Arc, RwLock, Weak};


use crate::types::SampleType;

pub struct OutPort {
    /// The port's ID label
    label: String,

    /// A weak pointer to an output port's value
    value: Arc<RwLock<Option<SampleType>>>,
}

impl OutPort {
    pub fn new(label: String) -> Self {
        // Will be initialized upon first process
        let value = Arc::new(RwLock::new(None));

        Self {
            label,
            value,
        }
    }

    pub fn get_label(&self) -> &str {
        &self.label
    }

    pub fn set_label(&mut self, new_label: String) {
        self.label = new_label;
    }

    pub fn set_value(&self, new: f64) {
        let mut value = self.value.write().expect("RwLock is poisoned");
        *value = Some(new);
    }

    pub fn get_ref(&self) -> Weak<RwLock<Option<SampleType>>> {
        Arc::downgrade(&self.value)
    }
}
