//! # rye-animations
//!
//! Declarative animations for rye — transitions, spring physics, shared element transitions.

#![deny(missing_docs)]

pub mod transition;
pub mod spring;

pub use transition::Transition;
pub use spring::Spring;
