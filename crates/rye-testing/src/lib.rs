//! # rye-testing
//!
//! Testing utilities for rye — virtual renderer, event simulation, query helpers, snapshots.

pub mod advanced;
pub mod events;
pub mod integration;
pub mod queries;
pub mod test_renderer;

pub use advanced::{
    A11yNode, ComponentContract, ContractProp, E2eTestConfig, EquivalenceResult, FuzzGenerator,
    FuzzResult, GeneratedTest, PerfBaseline, PerfBenchmark, PerfCheckResult, PlaywrightBrowser,
    RenderPlatform, SemanticDiff, SemanticNode, SignalGraph, SignalUpdate, TestSelector,
    TraceEvent,
};
pub use integration::{
    IntegrationTestCase, IntegrationTestRunner, MockSsrServer, TestRequest, TestResponse,
};
pub use test_renderer::{TestElement, TestNode, TestNodeKind, TestRenderer, TestText};
