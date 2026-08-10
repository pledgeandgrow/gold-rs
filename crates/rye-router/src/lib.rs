//! # rye-router
//!
//! Type-safe routing for rye — nested routes, guards, lazy loading, SSR-aware.
//!
//! **STUB** — This crate is a placeholder. Types are defined but routing
//! logic (matching, guards, lazy loading, SSR integration) is not implemented.

#![deny(missing_docs)]

pub mod link;
pub mod route;
pub mod router;

pub use route::{Route, RouteMatch};
pub use router::Router;
