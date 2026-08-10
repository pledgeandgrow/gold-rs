//! # rye-html
//!
//! DOM renderer for rye — web-sys based renderer for WASM target.

#![deny(missing_docs)]

#[cfg(target_arch = "wasm32")]
pub mod batch;
#[cfg(target_arch = "wasm32")]
pub mod dom_renderer;
pub mod events;
pub mod hydrate;
pub use hydrate::{hydrate, HydrationTarget};
pub mod js_interop;
pub mod web_components;

#[cfg(target_arch = "wasm32")]
pub use batch::{apply_mutation_direct, apply_mutations, DomMutation};
#[cfg(target_arch = "wasm32")]
pub use dom_renderer::DomRenderer;
pub use js_interop::{
    import_js, js_interop_script, JsArrayBuilder, JsModule, JsObjectBuilder, JsValue,
};
pub use web_components::{define_component_script, CustomElement, WebComponentDef};
