//! # rye-core
//!
//! Core abstractions for the rye UI framework.
//!
//! Contains the `Component` trait, `Renderer` trait, `Element` type,
//! and the template engine that connects components to renderers.

pub mod ai;
pub mod alloc;
pub mod code_split;
pub mod component;
pub mod component_codegen;
pub mod component_registry;
pub mod context;
pub mod cross_tab;
pub mod dual_pass;
pub mod element;
pub mod element_hydration;
pub mod error_codes;
pub mod event_delegation;
pub mod hooks;
pub mod houdini;
pub mod hydration;
pub mod incremental_hydration;
pub mod interop;
pub mod islands;
pub mod layout_cache;
pub mod offscreen;
pub mod perf;
pub mod platform;
pub mod reconcile;
pub mod render_hooks;
pub mod render_loop;
pub mod render_props;
pub mod render_to_texture;
pub mod renderer;
pub mod rendering;
pub mod retry_boundary;
pub mod server_action;
pub mod shadow_dom;
pub mod static_template;
pub mod suspense;
pub mod template;
pub mod testing;
pub mod text_shaping_cache;
pub mod tooling;
pub mod tools;
pub mod url_state;

pub use alloc::RenderArena;
pub use code_split::{
    chunk_loader_script, init_loader, is_chunk_loaded, load_chunk, ChunkLoader, ChunkStatus,
    LazyComponent, LoadedChunk,
};
pub use component::{Component, ComponentProps, FunctionComponent};
pub use component_codegen::{
    ComponentCodeGenerator, ComponentGenDef, PropDef, PropType, TemplatePart,
};
pub use context::{provide_context, provide_context_signal, use_context, use_context_signal};
pub use element::Element;
pub use event_delegation::EventDelegator;
pub use hooks::{use_signal, use_signal_default};
pub use hydration::{
    generate_marker, hydrate, parse_markers, HydrationKind, HydrationMarker, HydrationPlan,
    HydrationResult,
};
pub use incremental_hydration::{HydrationPriority, HydrationTask, IncrementalHydrationScheduler};
pub use islands::{
    bootstrap_script, init_registry, render_island, HydrationStrategy, Island, IslandMeta,
    IslandRegistry,
};
pub use layout_cache::{LayoutCache, LayoutConfig, LayoutResult, SmartLayoutCache};
pub use platform::{NoopPlatform, Platform, PlatformError, PlatformResult, RenderBackend};
pub use render_loop::{hydrate_to_dom, mount, render_tree_to_string, RenderScope};
pub use renderer::{BatchRenderer, EventHandler, Hydratable, NodeId, Renderer};
pub use server_action::{
    call_server, invoke_action, list_actions, register_action, set_transport, InProcessTransport,
    ServerError, ServerTransport,
};
pub use suspense::{ErrorBoundary, Suspense, SuspenseState};
pub use template::{
    shared_event_handler, ReactiveFn, ReactiveListFn, ReactiveValue, SharedEventHandler, Template,
    TemplateNode,
};
pub use text_shaping_cache::{ShapedGlyph, ShapedText, ShapingKey, TextShapingCache};
