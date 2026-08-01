# Rendering Strategy — Hybrid Compile-Time + Fine-Grained Signals

> Goal 3: Choose and document the rendering strategy for `rye`.

---

## Decision: Hybrid Model

We adopt a **hybrid rendering strategy** that combines:

1. **Compile-time template analysis** (like Svelte/Solid) — static template parts are generated once at build time
2. **Fine-grained signal-based reactivity** (like SolidJS/Leptos) — only DOM nodes that depend on changed state update
3. **Optional VDOM mode** (like React/Dioxus) — for components where full diffing is preferable (e.g., complex conditional trees)

### Why Hybrid?

| Approach | Pros | Cons |
|---|---|---|
| **VDOM only** (React) | Familiar, simple mental model, easy SSR | Overkill for most updates, re-renders whole components |
| **Fine-grained only** (Solid) | Maximum performance, minimal updates | Harder SSR, complex internal bookkeeping |
| **Compile-time only** (Svelte) | Smallest bundles, fastest | Compiler lock-in, limited flexibility |
| **Hybrid** (our choice) | Best of all worlds | More complex to implement |

The hybrid approach lets us:
- Use fine-grained signals for **95% of updates** (maximum performance)
- Fall back to VDOM diffing for **complex dynamic structures** where signal tracking is impractical
- Compile templates to **static + dynamic parts** (small bundles, fast initialization)
- Support **SSR** cleanly (render to string, hydrate with signals)

---

## How It Works

### Compile-Time Phase

The `template!` macro analyzes the template at compile time and separates it into:

1. **Static template** — the fixed structure (tags, static attributes, static text). Compiled into a `Template` struct that is created once and reused.
2. **Dynamic bindings** — the parts that depend on reactive state (dynamic text, dynamic attributes, conditional blocks, loops). Each binding is linked to a signal.

```rust
template! {
    div {
        class: "card",                    // static
        h1 { "User Profile" }             // static
        p { "Name: " {user.name()} }      // static text + dynamic binding
        if user.is_admin() {              // dynamic conditional
            span { class: "badge", "Admin" }
        }
        For each(item in items) {         // dynamic list
            li { key: item.id, {item.label()} }
        }
    }
}
```

This compiles to:
- A static `Template` with placeholders for dynamic parts
- Signal subscriptions wired to specific DOM nodes
- No VDOM diffing needed for this template

### Runtime Phase

When a signal changes:
1. The signal notifies its subscribers (effects, memos, DOM bindings)
2. Each subscriber updates **only its target** (a text node, an attribute, a conditional block)
3. No component re-render is triggered
4. No VDOM diff is performed
5. Updates are batched within a microtask

### VDOM Fallback Mode

For cases where fine-grained tracking is impractical (e.g., highly dynamic component trees, plugin-rendered content), a component can opt into VDOM mode:

```rust
#[component(vdom)]
fn DynamicList(props: DynamicListProps) -> Element {
    // This component uses VDOM diffing instead of fine-grained signals
    template! { ... }
}
```

This is the exception, not the default. Most components should use the default fine-grained mode.

---

## Renderer Trait

The core rendering abstraction is the `Renderer` trait:

```rust
pub trait Renderer: 'static {
    type Node: Clone;
    type Text: Clone;
    type Element: Clone;

    fn create_element(&mut self, tag: &str) -> Self::Element;
    fn create_text(&mut self, content: &str) -> Self::Text;
    fn set_text(&mut self, node: &Self::Text, content: &str);
    fn set_attribute(&mut self, el: &Self::Element, name: &str, value: &str);
    fn remove_attribute(&mut self, el: &Self::Element, name: &str);
    fn insert_child(&mut self, parent: &Self::Element, child: &Self::Node, index: usize);
    fn remove_child(&mut self, parent: &Self::Element, index: usize);
    fn replace_child(&mut self, parent: &Self::Element, new: &Self::Node, index: usize);
    fn set_event_listener(&mut self, el: &Self::Element, event: &str, handler: EventHandler);
    fn remove_event_listener(&mut self, el: &Self::Element, event: &str);
}
```

### Implementations

| Renderer | Target | Node Type | Notes |
|---|---|---|---|
| `DomRenderer` | Web (WASM) | `web_sys::Node` | Uses `web-sys`, batched DOM writes |
| `SsrRenderer` | Server | `String` | Streaming via async, hydration markers |
| `NativeRenderer` | Desktop/Mobile | `RenderNode` (GPU) | Via `wgpu`, custom layout engine |
| `TestRenderer` | Testing | `TestNode` | In-memory, queryable, no browser needed |
| `WebViewRenderer` | Fallback | `web_sys::Node` | WebView for platforms without GPU |

---

## Performance Targets

| Metric | Target | Benchmark |
|---|---|---|
| Initial render (hello world) | <5ms | vs React ~15ms, Solid ~3ms |
| Update (single signal) | <1ms | vs React ~5ms (re-render), Solid ~0.5ms |
| Update (list of 1000 items) | <10ms | vs React ~50ms, Solid ~8ms |
| WASM bundle (hello world) | <50KB gzipped | vs Dioxus ~80KB, Yew ~100KB |
| SSR throughput | >100k req/s | vs Next.js ~10k req/s |
| Memory per component | <1KB | vs React ~2KB |

---

## SSR + Hydration Strategy

1. **Server render**: `SsrRenderer` renders the component tree to HTML string with embedded hydration markers (`data-rye-id`, `data-rye-signal`)
2. **Client hydration**: `DomRenderer` reads the markers, attaches signal subscriptions to existing DOM nodes without re-creating them
3. **Progressive hydration**: Components hydrate as they become visible (with `IntersectionObserver` on web) or in priority order
4. **Streaming**: `<Suspense>` boundaries stream HTML as async data resolves. Client hydrates completed sections immediately.

---

## Key Differentiators from Competitors

| Feature | React | Vue | Solid | Dioxus | `rye` |
|---|---|---|---|---|---|
| Default rendering | VDOM | VDOM | Fine-grained | VDOM | Fine-grained |
| VDOM fallback | N/A | N/A | No | N/A | Yes (opt-in) |
| Compile-time templates | No | Partial | Yes | Yes | Yes |
| Native GPU renderer | No | No | No | No | Yes |
| SSR + hydration | Yes | Yes | Yes (Solid Start) | Yes | Yes (streaming) |
| Test renderer | No | No | No | No | Yes (built-in) |

---

*This document defines the rendering architecture. **Implemented** across `rye-core/src/renderer.rs`, `rye-core/src/reconcile.rs`, `rye-html/src/dom_renderer.rs`, `rye-desktop` (native GPU), `rye-ssr` (SSR), and `rye-testing` (test renderer).*
