//! Native tooling modules — Goals 147–150.
//!
//! - **Goal 147**: Template hot reload (HMR for templates)
//! - **Goal 148**: `rye inspect` CLI command
//! - **Goal 149**: Mobile build (iOS/Android)
//! - **Goal 150**: Deploy pipeline

pub mod deploy;
pub mod hot_reload;
pub mod inspect;
pub mod mobile;
