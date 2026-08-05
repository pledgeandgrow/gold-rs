//! # rye-html
//!
//! DOM renderer for rye — web-sys based renderer for WASM target.

#![deny(missing_docs)]

#[cfg(target_arch = "wasm32")]
pub mod dom_renderer;
#[cfg(target_arch = "wasm32")]
pub mod batch;
pub mod events;
pub mod hydrate;
pub use hydrate::{hydrate, HydrationTarget};
pub mod web_components;
pub mod js_interop;

#[cfg(target_arch = "wasm32")]
pub use dom_renderer::DomRenderer;
#[cfg(target_arch = "wasm32")]
pub use batch::{DomMutation, apply_mutations, apply_mutation_direct};
pub use web_components::{CustomElement, WebComponentDef, define_component_script};
pub use js_interop::{JsValue, JsModule, JsObjectBuilder, JsArrayBuilder, import_js, js_interop_script};
