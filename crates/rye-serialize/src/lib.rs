//! # rye-serialize
//!
//! Minimal serializer for rye — SSR state transfer without serde overhead.

#![deny(missing_docs)]

pub mod serializer;

pub use serializer::deserialize;
pub use serializer::serialize;
pub use serializer::{Deserialize, Serialize};
