//! # rye-testing
//!
//! Testing utilities for rye — virtual renderer, event simulation, query helpers, snapshots.

pub mod test_renderer;
pub mod queries;
pub mod events;
pub mod integration;
pub mod advanced;

pub use test_renderer::{TestRenderer, TestNode, TestElement, TestText, TestNodeKind};
pub use integration::{MockSsrServer, TestRequest, TestResponse, IntegrationTestCase, IntegrationTestRunner};
pub use advanced::{
    E2eTestConfig, PlaywrightBrowser, TestSelector,
    ComponentContract, ContractProp,
    PerfBenchmark, PerfBaseline, PerfCheckResult,
    SemanticNode, SemanticDiff,
    FuzzGenerator, FuzzResult,
    A11yNode,
    RenderPlatform, EquivalenceResult,
    SignalGraph, SignalUpdate,
    GeneratedTest, TraceEvent,
};
