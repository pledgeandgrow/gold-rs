# Signal / Primitive Reactivity Crate Design

> Goal 12 — Implement `Signal<T>`, `Memo<T>`, `Effect`, `Resource<T>` with automatic tracking, no hook rules, batched updates, and `Send + Sync` support.

---

## Design Goals

- **No rules of hooks** — Signals can be created anywhere, conditionally, in loops
- **Automatic dependency tracking** — No manual `useMemo`/`useCallback`
- **Batched updates by default** — Multiple signal writes = one notification cycle
- **Borrow-checker friendly** — `Read`/`Write` split to avoid `RefCell` borrow conflicts
- **Thread-safe option** — `Send + Sync` variants for multi-threaded runtimes (SSR, native)
- **Zero-cost reads** — Reading a signal outside a tracking scope has no overhead

---

## Core Primitives

### Signal<T> — Reactive State

```rust
/// A reactive value. Writes notify subscribers. Reads in tracking
/// scopes register dependencies.
pub struct Signal<T: 'static> { /* ... */ }

impl<T: Clone + 'static> Signal<T> {
    /// Create a new signal.
    pub fn new(value: T) -> Self;

    /// Read the current value. Registers dependency if in tracking scope.
    pub fn get(&self) -> T;

    /// Set a new value. Notifies subscribers (batched if inside `batch()`).
    pub fn set(&self, value: T);

    /// Functional update.
    pub fn update<F: FnOnce(&mut T)>(&self, f: F);

    /// Read without tracking — no dependency registered.
    pub fn get_untracked(&self) -> T;

    /// Set without notifying subscribers.
    pub fn set_untracked(&self, value: T);
}
```

**Read/Write split:**

To avoid borrow checker issues (e.g., reading a signal while writing another in the same scope), we provide split handles:

```rust
let count = Signal::new(0);

// Read handle — can be cloned, passed around
let reader = count.read_only();
// Write handle — can be cloned, passed around
let writer = count.write_only();

// In a component:
let count = Signal::new(0);
let count_reader = count.read_only();
let count_writer = count.write_only();

button {
    onclick: move |_| count_writer.set(count_reader.get() + 1),
    "Increment"
}
```

### Memo<T> — Derived State

```rust
/// A computed value that re-calculates when dependencies change.
/// Results are cached until a dependency changes.
pub struct Memo<T: Clone + 'static> { /* ... */ }

impl<T: Clone + 'static> Memo<T> {
    pub fn new<F: Fn() -> T + 'static>(compute: F) -> Self;
    pub fn get(&self) -> T;
    pub fn get_untracked(&self) -> T;
}
```

**Example:**
```rust
let first = Signal::new("Jane");
let last = Signal::new("Doe");

// Re-computes when first or last changes
let full = Memo::new(move || format!("{} {}", first(), last()));

assert_eq!(full(), "Jane Doe");
first.set("John");
assert_eq!(full(), "John Doe");
```

### Effect — Side Effects

```rust
/// A side effect that re-runs when dependencies change.
/// Runs immediately on creation, then re-runs on dependency changes.
pub struct Effect { /* ... */ }

impl Effect {
    pub fn new<F: Fn() + 'static>(callback: F) -> Self;
}

/// Register a cleanup function for the current scope.
/// Runs before the effect re-runs or when the component unmounts.
pub fn on_cleanup<F: FnOnce() + 'static>(cleanup: F);
```

**Example:**
```rust
let count = Signal::new(0);

Effect::new(move || {
    let c = count();
    log::info!("Count: {}", c);

    on_cleanup(|| {
        log::info!("Cleaning up effect for count={}", c);
    });
});

count.set(1); // logs "Count: 1", then "Cleaning up..." from previous
```

### Resource<T> — Async Data

```rust
/// Async data that re-fetches when dependencies change.
/// Automatically cancels previous fetch on re-fetch.
pub struct Resource<T: Clone + 'static> { /* ... */ }

pub enum ResourceState<T> {
    Pending,
    Ready(T),
    Error(String),
}

impl<T: Clone + 'static> Resource<T> {
    pub fn new<F, Fut>(fetch: F) -> Self
    where
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = Result<T, String>> + 'static;

    pub fn get(&self) -> ResourceState<T>;
    pub fn refresh(&self);
}
```

**Example:**
```rust
let user_id = Signal::new(1);

let user = Resource::new(move || {
    let id = user_id();
    async move {
        fetch_user(id).await
    }
});

// In template:
match user() {
    ResourceState::Pending => div { "Loading..." },
    ResourceState::Ready(u) => div { {u.name} },
    ResourceState::Error(e) => div { class: "error", {e} },
}
```

---

## Dependency Tracking

### How it works

```
Tracking Scope (Effect/Memo/template binding)
│
├── Signal::get() called
│   └── Registers dependency: "this scope reads this signal"
│
├── Signal::set() called (later)
│   └── Notifies all registered scopes
│       └── Scope re-runs (Effect/Memo re-executes, template re-renders)
│
└── Scope finishes
    └── Dependency list is finalized for this run
    └── Next run starts with fresh dependency list
```

### Implementation

```rust
thread_local! {
    static CURRENT_SCOPE: RefCell<Option<ScopeRef>> = const { RefCell::new(None) };
}

struct ScopeRef {
    dependencies: Vec<SignalId>,
    callback: Box<dyn Fn()>,
}

/// Enter a tracking scope. Any signal reads inside the closure
/// register as dependencies.
pub fn with_tracking<F, R>(callback: &ScopeRef, f: F) -> R
where F: FnOnce() -> R
{
    CURRENT_SCOPE.with(|s| {
        *s.borrow_mut() = Some(callback.clone());
    });
    let result = f();
    CURRENT_SCOPE.with(|s| {
        *s.borrow_mut() = None;
    });
    result
}
```

---

## Batched Updates

```rust
/// Batch multiple signal writes. Subscribers are notified once
/// after the closure completes, not on each write.
pub fn batch<F: FnOnce() -> R, R>(f: F) -> R;

/// Check if currently inside a batch.
pub fn is_batching() -> bool;
```

**Example:**
```rust
let a = Signal::new(1);
let b = Signal::new(2);

// Without batching: 2 notification cycles
a.set(10);  // notifies
b.set(20);  // notifies

// With batching: 1 notification cycle
batch(|| {
    a.set(10);  // queued
    b.set(20);  // queued
    // Subscribers notified once here
});
```

**Automatic batching:**
- Event handlers are automatically wrapped in `batch()`
- `Effect` callbacks are automatically batched
- Only manual signal writes outside these contexts trigger immediate notification

---

## Send + Sync Support

For SSR and native multi-threaded runtimes, we provide thread-safe variants:

```rust
/// Thread-safe signal for multi-threaded contexts (SSR, native).
/// Uses RwLock instead of RefCell.
pub struct SyncSignal<T: Send + Sync + 'static> { /* ... */ }

impl<T: Clone + Send + Sync + 'static> SyncSignal<T> {
    pub fn new(value: T) -> Self;
    pub fn get(&self) -> T;
    pub fn set(&self, value: T);
}
```

**When to use which:**
- `Signal<T>` — WASM (single-threaded), uses `RefCell` (zero overhead)
- `SyncSignal<T>` — SSR/native (multi-threaded), uses `RwLock` (small overhead)

The `rye::prelude` exports `Signal` for WASM and `SyncSignal` behind a `ssr` feature flag.

---

## Hook Functions

These are thin wrappers around signal primitives, designed for ergonomics:

```rust
/// Create a signal in the current component scope.
/// Equivalent to Signal::new() but auto-cleaned on unmount.
pub fn use_signal<T: 'static>(initial: impl FnOnce() -> T) -> Signal<T>;

/// Create a memo in the current component scope.
pub fn use_memo<T: Clone + 'static>(compute: impl Fn() -> T + 'static) -> Memo<T>;

/// Create an effect in the current component scope.
pub fn use_effect<F: Fn() + 'static>(callback: F);

/// Create a resource in the current component scope.
pub fn use_resource<T, F, Fut>(fetch: F) -> Resource<T>
where
    T: Clone + 'static,
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = Result<T, String>> + 'static;

/// Create a non-reactive reference (like useRef in React).
pub fn use_ref<T: 'static>(initial: impl FnOnce() -> T) -> Rc<RefCell<T>>;
```

**No rules of hooks:** These can be called conditionally, in loops, anywhere — because they're just signal constructors, not positional hooks.

```rust
// This is VALID in rye (would break React):
#[component]
fn Search(props) {
    let results = if props.advanced {
        use_signal(|| Vec::new())  // conditional signal creation
    } else {
        Signal::new(Vec::new())
    };

    for i in 0..props.filter_count {
        use_effect(move || { /* ... */ });  // in a loop
    }
}
```

---

## Comparison with Competitors

| Feature | React | Vue | SolidJS | Leptos | rye |
|---|---|---|---|---|---|
| State primitive | useState | ref/reactive | createSignal | create_signal | Signal::new |
| Derived | useMemo | computed | createMemo | create_memo | Memo::new |
| Side effect | useEffect | watchEffect | createEffect | create_effect | Effect::new |
| Async | (manual) | (manual) | createResource | Resource | Resource::new |
| Hook rules | Yes (strict) | No | No | No | No |
| Auto-tracking | No (deps array) | Yes | Yes | Yes | Yes |
| Batched | Yes (automatic) | Yes | Yes | Yes | Yes (automatic) |
| Thread-safe | N/A (JS) | N/A (JS) | N/A (JS) | Optional | Optional (SyncSignal) |
| Borrow-friendly | N/A | N/A | N/A | No (RefCell) | Yes (Read/Write split) |

---

*This document defines the reactivity crate design. **Implemented** in `rye-signals` crate — `signal.rs` (`Signal<T>` with read/write split), `memo.rs` (`Memo<T>`), `effect.rs` (`Effect`, `on_cleanup`), `resource.rs` (`Resource<T>`, `ResourceState`), `global.rs` (`GlobalSignal`), `batch.rs` (batched updates), `runtime.rs` (dependency tracking, `SyncSignal` for SSR).*
