use std::sync::{RwLock, Weak};

pub struct InPort {
    /// The port's ID label
    label: String,

    /// A weak pointer to an output port's value
    value: Weak<RwLock<Option<f64>>>,

    /// This is a suggested lower bound on the port's value.
    /// A user is free to ignore this.
    lower_range: f64,

    /// This is a suggested upper bound on the port's value.
    /// A user is free to ignore this.
    upper_range: f64,

    /// A default value, in case it's not connected, i.e., it's value is None
    default: f64,
}

impl InPort {
    pub fn new(
        label: String,
        lower_range: f64,
        upper_range: f64,
        default: f64,
        ) -> Self
    {
        let value = Weak::new();

        Self {
            label,
            value,
            lower_range,
            upper_range,
            default,
        }
    }

    pub fn get_label(&self) -> &str {
        &self.label
    }

    pub fn set_label(&mut self, new_label: String) {
        self.label = new_label;
    }

    pub fn get_value(&self) -> f64 {
        match self.value.upgrade() {
            Some(v) => {
                let v = v.read().expect("RwLock is poisoned").unwrap_or(self.default);
                v
            },
            None => self.default,
        }
    }

    pub fn set_value(&mut self, value: Weak<RwLock<Option<f64>>>) {
        self.value = value;
    }

    pub fn get_lower_range(&self) -> f64 {
        self.lower_range
    }

    pub fn set_lower_range(&mut self, new_lower_range: f64) {
        self.lower_range = new_lower_range;
    }

    pub fn get_upper_range(&self) -> f64 {
        self.upper_range
    }

    pub fn set_upper_range(&mut self, new_upper_range: f64) {
        self.upper_range = new_upper_range;
    }

    pub fn is_connected(&self) -> bool {
        self.value.strong_count() > 0
    }
}
