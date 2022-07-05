use crate::rack::Rack;

/// A Project consists of a Rack, a clock source for the Rack
/// to follow and other Project metadata (TODO)
pub struct Project {
    clock: f64,
    rack: Rack,
}

