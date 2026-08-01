//! # rye-router
//!
//! Type-safe routing for rye — nested routes, guards, lazy loading, SSR-aware.

#![deny(missing_docs)]

pub mod route;
pub mod router;
pub mod link;

pub use route::{Route, RouteMatch};
pub use router::Router;
