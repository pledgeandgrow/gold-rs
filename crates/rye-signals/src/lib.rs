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


pub mod signal;
pub mod memo;
pub mod effect;
pub mod resource;
pub mod global;
pub mod batch;
pub mod selector;
pub mod prune;
pub mod snapshot;
pub mod saga;
pub mod optimistic;
pub mod persistence;
pub mod debounce;
pub mod priority_batch;
mod runtime;

pub use signal::{Signal, ReadSignal, WriteSignal};
pub use memo::Memo;
pub use effect::{Effect, on_cleanup};
pub use resource::{Resource, ResourceState};
pub use global::GlobalSignal;
pub use batch::batch;
pub use selector::{Selector, select};
pub use saga::{Saga, SagaBuilder, SagaStep, SagaState, StepResult};
pub use optimistic::{optimistic_update, optimistic_update_sync, OptimisticUpdate, OptimisticResult};
pub use persistence::{persist, PersistedSignal, PersistenceStrategy, MemoryPersistence, NoopPersistence, CustomPersistence, PersistenceType};
pub use debounce::{debounced, throttled, Debounced, Throttled};
pub use priority_batch::{Priority, batch_high, batch_normal, batch_low};
