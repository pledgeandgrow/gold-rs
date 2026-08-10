//! # rye-signals
//!
//! Fine-grained reactive signals for rye.
//!
//! Core primitives:
//! - `Signal<T>` — reactive state
//! - `Memo<T>` — derived/computed state
//! - `Effect` — side effects with automatic dependency tracking
//! - `Resource<T>` — async data with automatic cancellation
//! - `GlobalSignal<T>` — app-wide reactive state
//! - `Selector<T>` — derived state with structural sharing (goal 166)
//! - `Saga<T, E>` — multi-step async with compensation (goal 169)
//! - `Debounced<T>` / `Throttled<T>` — rate-limited computed signals (goal 172)

pub mod batch;
pub mod debounce;
pub mod effect;
pub mod global;
pub mod memo;
pub mod optimistic;
pub mod persistence;
pub mod priority_batch;
pub mod prune;
pub mod resource;
mod runtime;
pub mod saga;
pub mod selector;
pub mod signal;
pub mod snapshot;

pub use batch::batch;
pub use debounce::{debounced, throttled, Debounced, Throttled};
pub use effect::{on_cleanup, Effect};
pub use global::GlobalSignal;
pub use memo::Memo;
pub use optimistic::{
    optimistic_update, optimistic_update_sync, OptimisticResult, OptimisticUpdate,
};
pub use persistence::{
    persist, CustomPersistence, MemoryPersistence, NoopPersistence, PersistedSignal,
    PersistenceStrategy, PersistenceType,
};
pub use priority_batch::{batch_high, batch_low, batch_normal, Priority};
pub use resource::{Resource, ResourceState};
pub use saga::{Saga, SagaBuilder, SagaState, SagaStep, StepResult};
pub use selector::{select, Selector};
pub use signal::{ReadSignal, Signal, WriteSignal};
