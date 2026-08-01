# Reactivity Model — Signal-Based Fine-Grained Reactivity

> Goal 5: Define and document the reactivity model for `rye`.

---

## Decision: Signal-Based Reactivity with Automatic Dependency Tracking

We adopt a **signal-based reactivity model** inspired by SolidJS, Vue's reactivity, and Leptos signals, with key improvements:

1. **No "rules of hooks"** — signals can be created anywhere, conditionally, in loops
2. **Automatic dependency tracking** — no manual dependency arrays
3. **Batched updates by default** — multiple signal writes in one tick = one update pass
4. **Send + Sync support** — signals work in multi-threaded environments (SSR, native)
5. **Zero-cost reads** — reading a signal is a function call, no allocation

---

## Core Primitives

### `Signal<T>` — Reactive State

```rust
// Create a signal
let count = Signal::new(0);

// Read (tracks dependency if inside an effect/memo)
let current = count();           // or count.get()

// Write (triggers subscribers)
count.set(5);                    // or *count.write() = 5;

// Update (functional update)
count.update(|v| *v + 1);
```

**Key properties:**
- `Signal<T>` is `Clone` (inner shared state via `Rc<RefCell>` on web, `Arc<Mutex>` on native)
- Reading inside a tracking scope (Effect, Memo, template binding) auto-subscribes
- Reading outside a tracking scope just returns the current value
- Writes are batched — all subscribers notified once per tick

### `Memo<T>` — Derived/Computed State

```rust
let count = Signal::new(0);
let doubled = Memo::new(move || count() * 2);
let quadrupled = Memo::new(move || doubled() * 2);  // chains work

// Reading
assert_eq!(quadrupled(), 0);
count.set(5);
assert_eq!(quadrupled(), 20);  // automatically recomputed
```

**Key properties:**
- Only recomputes when dependencies change
- Result is cached — subsequent reads return cached value
- Dependencies are tracked automatically (no manual arrays)
- Memos can chain — dependency graph is built dynamically

### `Effect` — Side Effects

```rust
let count = Signal::new(0);

Effect::new(move || {
    println!("Count changed to: {}", count());
});
// Immediately runs once, then re-runs whenever count changes

count.set(1);   // prints "Count changed to: 1"
count.set(2);   // prints "Count changed to: 2"
```

**Key properties:**
- Runs immediately on creation
- Re-runs when any read signal changes
- Automatic cleanup — if the effect returns a cleanup function, it's called before re-run
- Tied to component lifecycle — destroyed when component unmounts

### `Resource<T>` — Async Data

```rust
let user_id = Signal::new(1);

let user = Resource::new(move || {
    let id = user_id();
    async move { fetch_user(id).await }
});

// In template:
template! {
    Suspense {
        fallback: template! { "Loading..." },
        {match user() {
            ResourceState::Pending => template! { "Loading..." },
            ResourceState::Ready(user) => template! { h1 { {user.name} } },
            ResourceState::Error(err) => template! { p { "Error: " {err} } },
        }}
    }
}
```

**Key properties:**
- Automatically re-fetches when signal dependencies change
- Cancels in-flight request when dependencies change (via `AbortHandle`)
- Returns `Pending | Ready(T) | Error(E)` state
- Integrates with `<Suspense>` for declarative loading states
- SSR-aware — resolves on server, serializes state, hydrates on client

### `GlobalSignal<T>` — App-Wide State

```rust
static THEME: GlobalSignal<String> = Signal::global(|| "light".to_string());

// Any component can read or write
fn Header() -> Element {
    template! {
        button {
            onclick: move |_| *THEME.write() = if *THEME.read() == "light" { "dark" } else { "light" },
            "Toggle Theme"
        }
    }
}
```

**Key properties:**
- No context provider needed — just declare and use
- Thread-safe (`Arc<Mutex>` internally)
- Reactive — changes propagate to all readers
- Devtools integration — visible in signal inspector

---

## Component-Scoped Hooks

These are convenience wrappers around signals/memos/effects that are tied to the component lifecycle:

```rust
fn Counter() -> Element {
    // use_signal — creates a Signal owned by this component
    let count = use_signal(|| 0);

    // use_memo — creates a Memo owned by this component
    let doubled = use_memo(move || count() * 2);

    // use_effect — creates an Effect tied to this component's lifecycle
    use_effect(move || {
        println!("Count: {}", count());
    });

    // use_resource — creates a Resource tied to this component
    let data = use_resource(move || async { fetch_data(count()).await });

    template! {
        div {
            button { onclick: move |_| count += 1, "Increment" }
            p { "Count: " {count} " (doubled: " {doubled} ")" }
        }
    }
}
```

### Why "No Rules of Hooks"?

React's hooks must be called in the same order every render because React relies on call order to identify hooks. `rye` doesn't have this limitation because:

1. **Signals are not stored by call order** — each signal is an independent `Rc<RefCell<T>>` / `Arc<Mutex<T>>`
2. **Components don't "re-render"** — the function runs once, signals persist
3. **No hook dispatcher** — signals are just values, not framework-managed slots

This means all of these are valid:
```rust
fn MyComponent() -> Element {
    let condition = use_signal(|| true);

    // Conditional signal creation — VALID
    let value = if condition() {
        Some(use_signal(|| 0))
    } else {
        None
    };

    // Signal in a loop — VALID
    let items: Vec<_> = (0..3).map(|i| use_signal(move || i)).collect();

    // Early return before signal creation — VALID
    if !condition() {
        return template! { div { "Nothing" } };
    }
    let late_signal = use_signal(|| 42);

    template! { div { {late_signal} } }
}
```

---

## Update Batching

Multiple signal writes in the same tick are batched:

```rust
let a = Signal::new(1);
let b = Signal::new(2);

Effect::new(move || {
    println!("a={}, b={}", a(), b());
});
// Prints: a=1, b=2

batch(|| {
    a.set(10);
    b.set(20);
    a.set(100);  // second write to same signal — only final value matters
});
// Prints once: a=100, b=20 (NOT three times)
```

**Batching rules:**
- All writes within a `batch()` closure are collected
- Effects/memos are notified once after the batch completes
- If a signal is written multiple times in a batch, only the final value triggers notification
- Event handlers are automatically wrapped in a batch
- Manual batching via `batch(|| { ... })` for multi-signal updates

---

## Dependency Tracking — How It Works

```
Signal<T> ──read──> Tracking Scope (Effect/Memo/Binding)
                         │
                         │ registers as subscriber
                         ▼
                    Signal's subscriber list
                         │
                    Signal is written
                         │
                         ▼
                    Notify all subscribers
                         │
                         ▼
                    Re-run Effect / recompute Memo / update DOM node
```

1. When a signal is read inside a tracking scope, the scope registers itself as a subscriber
2. When the signal is written, it notifies all subscribers
3. Each subscriber re-runs, re-reading signals (which re-registers dependencies)
4. Old dependencies that are no longer read are automatically unsubscribed
5. This is all automatic — no manual dependency arrays

---

## Memory Management

| Environment | Signal Storage | Thread Safety |
|---|---|---|
| Web (WASM) | `Rc<RefCell<T>>` | Single-threaded (WASM is single-threaded) |
| Native (Desktop/Mobile) | `Arc<Mutex<T>>` | Multi-threaded |
| SSR | `Arc<Mutex<T>>` | Multi-threaded (tokio async runtime) |

Signals are reference-counted. When the last clone is dropped, the signal is deallocated. Effects tied to a component are automatically cleaned up when the component unmounts.

### Cleanup

```rust
use_effect(move || {
    let interval = setInterval(|| println!("tick"), 1000);

    // Return a cleanup function
    move || clearInterval(interval)
});
// Cleanup runs before re-run (if deps change) or on component unmount
```

---

## Comparison with Competitors

| Feature | React hooks | Vue Composition | Solid signals | Dioxus signals | `rye` signals |
|---|---|---|---|---|---|
| Rules of hooks | Yes (strict) | No | No | No | No |
| Auto dependency tracking | No (manual arrays) | Yes | Yes | Yes | Yes |
| Fine-grained updates | No (component re-render) | Partial | Yes | Partial | Yes |
| Global state | External (Zustand etc.) | External (Pinia) | Yes (createRoot) | Yes (GlobalSignal) | Yes (GlobalSignal) |
| Batched updates | Yes (automatic) | Yes (nextTick) | Yes (batch) | Yes | Yes (automatic + manual) |
| Async primitive |useEffect + manual | watchEffect + manual | createResource | use_resource | Resource + Suspense |
| Cleanup | return function | onScopeDispose | onCleanup | use_cleanup | automatic + manual |
| Thread-safe | N/A (JS) | N/A (JS) | N/A (JS) | Yes (Arc) | Yes (Arc) |

---

## API Summary

```rust
// Primitives
Signal::new(value)                    // create signal
signal() / signal.get()              // read (tracks)
signal.set(value)                     // write
signal.update(|v| new_value)          // functional update
*signal.write() = value               // direct write

Memo::new(move || expr)               // computed
memo() / memo.get()                   // read (tracks)

Effect::new(move || { ... })          // side effect
use_effect(move || { ... })           // component-scoped effect

Resource::new(move || async { ... })  // async data
resource()                            // read state (Pending/Ready/Error)

Signal::global(|| value)              // global signal

// Hooks (component-scoped)
use_signal(|| value)                  // = Signal::new, component-scoped
use_memo(move || expr)                // = Memo::new, component-scoped
use_resource(move || async { ... })   // = Resource::new, component-scoped
use_context::<T>()                    // type-safe context injection
use_ref::<T>()                        // direct DOM/native node ref

// Utilities
batch(|| { ... })                     // batch multiple writes
untrack(|| { ... })                   // read without tracking
on_cleanup(|| { ... })                // register cleanup for current scope
```

---

*This document defines the reactivity model. **Implemented** in `rye-signals` crate — `signal.rs`, `memo.rs`, `effect.rs`, `resource.rs`, `global.rs`, `batch.rs`, `runtime.rs`.*
