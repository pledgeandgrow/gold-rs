//! Spring — physics-based spring animations.

/// A spring animation that interpolates from one value to another
/// using spring physics (mass, stiffness, damping).
pub struct Spring<T> {
    // TODO: current value, target, config
    _marker: std::marker::PhantomData<T>,
}

impl<T> Spring<T> {
    /// Create a new spring with the given initial value.
    pub fn new(_initial: T) -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

/// Spring configuration — mass, stiffness, damping.
#[derive(Debug, Clone, Copy)]
pub struct SpringConfig {
    /// Mass of the spring (default: 1.0).
    pub mass: f64,
    /// Stiffness of the spring (default: 170.0).
    pub stiffness: f64,
    /// Damping of the spring (default: 26.0).
    pub damping: f64,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            mass: 1.0,
            stiffness: 170.0,
            damping: 26.0,
        }
    }
}
