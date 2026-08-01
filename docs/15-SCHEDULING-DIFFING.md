# Scheduling & Diffing Engine Design

> Goal 17 — Dual-mode rendering: VDOM diffing with keyed reconciliation OR direct signal-driven updates. Developer chooses per-component or auto-detect.

---

## Design Goals

- **Signal mode (default)** — Fine-grained updates, no VDOM, direct DOM/native manipulation
- **VDOM mode (optional)** — For complex list reordering or when diffing is more efficient
- **Auto-detect** — Framework chooses based on component structure
- **Keyed reconciliation** — Efficient list updates with stable keys
- **Batched scheduling** — Multiple signal changes = one render pass

---

## Two Rendering Modes

### Signal Mode (default, like SolidJS/Leptos)

```
Signal change → find affected bindings → update specific DOM node
```

- No VDOM tree
- No diffing
- Each dynamic binding directly updates its target node
- O(1) per signal update (just call the binding's update function)
- Best for: most components, simple state updates

```rust
#[component]
fn Counter() {
    let count = use_signal(|| 0);

    // Signal mode: {count} creates a direct binding
    // When count changes, only this text node updates
    div {
        h1 { "Count: " {count} }
        button { onclick: count += 1, "+" }
    }
}
```

### VDOM Mode (optional, like React/Dioxus)

```
Signal change → re-render component → diff old vs new VDOM → apply patches
```

- Full VDOM tree per component
- Diffing with keyed reconciliation
- Best for: complex list reordering, conditional rendering with many branches

```rust
#[component]
#[render_mode(vdom)]  // Opt into VDOM mode
fn TodoList(props: TodoListProps) {
    let todos = use_signal(|| props.initial_todos);

    // VDOM mode: entire div is re-rendered on signal change,
    // then diffed against previous render
    div {
        For each(todo in todos()) {
            key: todo.id,
            TodoItem { todo: todo.clone() }
        }
    }
}
```

### Auto-detect

```rust
#[component]
#[render_mode(auto)]  // Default
fn MyComponent() {
    // Framework analyzes the template:
    // - If template has mostly static content with few dynamic bindings → signal mode
    // - If template has complex For loops with many items → VDOM mode
    // - If template has deeply nested conditionals → VDOM mode
}
```

---

## Scheduling

### Update queue

When a signal changes, the update is not applied immediately. Instead, it's queued:

```rust
/// The global scheduler — manages pending updates.
pub struct Scheduler {
    /// Pending signal updates, deduplicated.
    pending_signals: HashSet<SignalId>,
    /// Pending effect re-runs.
    pending_effects: Vec<EffectId>,
    /// Whether we're in a batch.
    batching: bool,
}
```

### Update flow

```
1. Signal::set() called
   └─ If batching: add to pending_signals, return
   └─ If not batching: process immediately

2. batch() ends (or event handler returns)
   └─ Process all pending_signals
   └─ For each signal:
      └─ Find all subscribed bindings
      └─ Call each binding's update function
      └─ Binding updates the DOM/native node directly
   └─ Process all pending_effects
      └─ Re-run each effect
      └─ Effects may set more signals (go to step 1)

3. After all updates:
   └─ Run any scheduled cleanups
   └─ Notify devtools profiler
```

### Priority

```rust
/// Update priority — lower numbers run first.
pub enum UpdatePriority {
    /// High priority — user-visible state (e.g., input value).
    High = 0,
    /// Normal priority — most signal updates.
    Normal = 1,
    /// Low priority — non-critical UI (e.g., tooltips).
    Low = 2,
    /// Idle — can be deferred to next frame.
    Idle = 3,
}
```

---

## Keyed Reconciliation (For VDOM mode)

```rust
/// Reconcile a list of children with keyed diffing.
///
/// Algorithm:
/// 1. Build map of old keys → old nodes
/// 2. For each new child:
///    a. If key exists in old map → reuse node, mark as moved
///    b. If key is new → create new node
/// 3. Remove old nodes whose keys are not in new list
/// 4. Reorder moved nodes to match new order
///
/// This is O(n) for most cases, O(n log n) worst case.
pub fn reconcile_keyed<R: Renderer, K: Eq + Hash>(
    renderer: &mut R,
    parent: &R::Element,
    old_children: &[(K, R::Node)],
    new_children: &[(K, R::Node)],
) {
    let old_map: HashMap<&K, &R::Node> = old_children.iter().map(|(k, n)| (k, n)).collect();
    let new_keys: HashSet<&K> = new_children.iter().map(|(k, _)| k).collect();

    // Remove children that are no longer present
    for (i, (key, _)) in old_children.iter().enumerate().rev() {
        if !new_keys.contains(key) {
            renderer.remove_child(parent, i);
        }
    }

    // Insert/move children to match new order
    for (i, (key, node)) in new_children.iter().enumerate() {
        if let Some(_old) = old_map.get(key) {
            // Reuse — move to position i
            renderer.move_child(parent, /* from */, i);
        } else {
            // New — insert at position i
            renderer.insert_child(parent, node, i);
        }
    }
}
```

---

## Comparison with Competitors

| Feature | React | SolidJS | Leptos | Dioxus | rye |
|---|---|---|---|---|---|
| Default mode | VDOM | Signals | Signals | VDOM | Signals |
| Optional VDOM | No | No | No | No | Yes |
| Auto-detect | No | N/A | N/A | No | Yes |
| Keyed lists | Yes | Yes | Yes | Yes | Yes |
| Batched updates | Yes | Yes | Yes | Yes | Yes |
| Update priority | Yes (lanes) | No | No | No | Yes |

---

*This document defines the scheduling/diffing engine. **Implemented** in `rye-core/src/reconcile.rs` (keyed reconciliation, `reconcile_keyed`), `rye-signals/src/runtime.rs` (scheduler, update queue, batching), and `rye-signals/src/batch.rs` (automatic batching in event handlers).*
