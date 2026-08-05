//! # rye-core
//!
//! Core abstractions for the rye UI framework.
//!
//! Contains the `Component` trait, `Renderer` trait, `Element` type,
//! and the template engine that connects components to renderers.

pub mod component;
pub mod element;
pub mod renderer;
pub mod template;
pub mod context;
pub mod reconcile;
pub mod event_delegation;
pub mod suspense;
pub mod hydration;
pub mod alloc;
pub mod server_action;
pub mod islands;
pub mod code_split;
pub mod perf;
pub mod rendering;
pub mod tools;
pub mod testing;
pub mod tooling;
pub mod error_codes;
pub mod component_registry;
pub mod ai;
pub mod url_state;
pub mod cross_tab;
pub mod offscreen;
pub mod render_to_texture;
pub mod render_hooks;
pub mod shadow_dom;
pub mod houdini;
pub mod element_hydration;
pub mod render_props;
pub mod static_template;
pub mod dual_pass;
pub mod retry_boundary;
pub mod incremental_hydration;
pub mod component_codegen;
pub mod layout_cache;
pub mod text_shaping_cache;
pub mod interop;
pub mod render_loop;
pub mod hooks;

pub use component::{Component, ComponentProps, FunctionComponent};
pub use element::Element;
pub use renderer::{Renderer, BatchRenderer, EventHandler, NodeId, Hydratable};
pub use template::{Template, TemplateNode, SharedEventHandler, ReactiveFn, ReactiveValue, shared_event_handler};
pub use context::{provide_context, use_context, provide_context_signal, use_context_signal};
pub use event_delegation::EventDelegator;
pub use suspense::{Suspense, ErrorBoundary, SuspenseState};
pub use hydration::{HydrationPlan, HydrationMarker, HydrationKind, HydrationResult, hydrate, parse_markers, generate_marker};
pub use server_action::{ServerError, ServerTransport, InProcessTransport, register_action, invoke_action, call_server, set_transport, list_actions};
pub use islands::{Island, IslandRegistry, IslandMeta, HydrationStrategy, render_island, init_registry, bootstrap_script};
pub use alloc::RenderArena;
pub use code_split::{ChunkLoader, ChunkStatus, LoadedChunk, LazyComponent, load_chunk, is_chunk_loaded, init_loader, chunk_loader_script};
pub use incremental_hydration::{IncrementalHydrationScheduler, HydrationTask, HydrationPriority};
pub use component_codegen::{ComponentCodeGenerator, ComponentGenDef, PropDef, PropType, TemplatePart};
pub use layout_cache::{LayoutCache, LayoutConfig, LayoutResult, SmartLayoutCache};
pub use text_shaping_cache::{TextShapingCache, ShapingKey, ShapedText, ShapedGlyph};
pub use render_loop::{mount, hydrate_to_dom, RenderScope, render_tree_to_string};
pub use hooks::{use_signal, use_signal_default};
