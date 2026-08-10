//! # rye-animations
//!
//! Declarative animations for rye — transitions, spring physics, shared element transitions.
//!
//! **STUB** — This crate is a placeholder. Types are defined but animation
//! logic (spring physics, transitions, FLIP) is not implemented.

#![deny(missing_docs)]

pub mod spring;
pub mod transition;

pub use spring::Spring;
pub use transition::Transition;
