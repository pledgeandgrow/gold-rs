# Renderer Trait Design

> Goal 16 — A `Renderer` trait that abstracts DOM (WASM), native (GPU), SSR (string), and test (virtual) backends. The core never depends on a specific renderer.

---

## Design Goals

- **Backend-agnostic** — Core framework has zero knowledge of DOM, wgpu, or HTML strings
- **Minimal surface** — Only primitives that all backends can implement
- **Performant** — No virtual dispatch in hot paths (batch operations)
- **Extensible** — Third parties can implement custom renderers
- **Type-safe** — Each renderer has strongly-typed node handles

---

## Renderer Trait

```rust
/// The rendering backend abstraction.
///
/// Implementations:
/// - `DomRenderer` (web/WASM via web-sys)
/// - `NativeRenderer` (desktop/mobile via wgpu)
/// - `SsrRenderer` (server-side, outputs HTML string)
/// - `TestRenderer` (in-memory for testing)
///
/// The core framework calls only these methods.
/// All rendering logic is renderer-agnostic.
pub trait Renderer: 'static {
    /// The node type — a handle to a rendered node.
    type Node: Clone;
    /// The text node type.
    type Text: Clone;
    /// The element node type.
    type Element: Clone;

    // ── Node creation ─────────────────────────────────

    /// Create a new element node with the given tag name.
    fn create_element(&mut self, tag: &str) -> Self::Element;

    /// Create a new text node with the given content.
    fn create_text(&mut self, content: &str) -> Self::Text;

    // ── Node mutation ─────────────────────────────────

    /// Set the text content of a text node.
    fn set_text(&mut self, node: &Self::Text, content: &str);

    /// Set an attribute on an element.
    fn set_attribute(&mut self, el: &Self::Element, name: &str, value: &str);

    /// Remove an attribute from an element.
    fn remove_attribute(&mut self, el: &Self::Element, name: &str);

    // ── Tree manipulation ─────────────────────────────

    /// Insert a child node at a specific index in the parent's children.
    fn insert_child(&mut self, parent: &Self::Element, child: &Self::Node, index: usize);

    /// Remove the child at the given index from the parent.
    fn remove_child(&mut self, parent: &Self::Element, index: usize);

    /// Replace the child at the given index with a new node.
    fn replace_child(&mut self, parent: &Self::Element, new: &Self::Node, index: usize);

    /// Move a child from one index to another within the same parent.
    fn move_child(&mut self, parent: &Self::Element, from: usize, to: usize);

    // ── Events ────────────────────────────────────────

    /// Set an event listener on an element.
    fn set_event_listener(&mut self, el: &Self::Element, event: &str, handler: EventHandler);

    /// Remove an event listener from an element.
    fn remove_event_listener(&mut self, el: &Self::Element, event: &str);

    // ── Root ──────────────────────────────────────────

    /// Get the root node (mount point).
    fn root(&self) -> Self::Element;
}

/// An event handler callback — boxed for type erasure.
pub type EventHandler = Box<dyn FnMut(&dyn std::any::Any) + 'static>;
```

---

## Batch Operations (Optional Trait)

For renderers that benefit from batching (e.g., DOM renderer avoids multiple reflows):

```rust
/// Optional batch operations for optimized renderers.
pub trait BatchRenderer: Renderer {
    /// Begin a batch of operations. The renderer may defer
    /// layout/paint until `end_batch`.
    fn begin_batch(&mut self);

    /// End a batch and flush all pending operations.
    fn end_batch(&mut self);
}
```

The framework wraps render updates in `begin_batch`/`end_batch` when the renderer supports it.

---

## Renderer Implementations

### DomRenderer (WASM)

```rust
/// Web/DOM renderer using web-sys.
/// Node types are real DOM nodes.
impl Renderer for DomRenderer {
    type Node = web_sys::Node;
    type Text = web_sys::Text;
    type Element = web_sys::Element;
    // ... implementation using web_sys APIs
}
```

### NativeRenderer (wgpu)

```rust
/// Native GPU renderer using wgpu.
/// Node types are internal render tree nodes.
impl Renderer for NativeRenderer {
    type Node = RenderNode;      // Internal GPU node
    type Text = RenderText;      // Glyph atlas reference
    type Element = RenderElement; // Layout + paint node
    // ... implementation using wgpu + taffy
}
```

### SsrRenderer (Server)

```rust
/// Server-side renderer that outputs HTML strings.
/// Node types are string builders.
impl Renderer for SsrRenderer {
    type Node = String;           // HTML string
    type Text = String;           // Text content
    type Element = String;        // HTML element string
    // ... implementation builds HTML with hydration markers
}
```

### TestRenderer (Testing)

```rust
/// In-memory renderer for unit tests.
/// Node types are simple structs.
impl Renderer for TestRenderer {
    type Node = TestNode;         // In-memory node
    type Text = TestText;         // Text content
    type Element = TestElement;   // Tag + attrs + children
    // ... implementation stores everything in memory
}
```

---

## Node Conversion

Since each renderer has different node types, the framework uses a generic `Node<R>` wrapper:

```rust
/// A renderer-specific node.
pub enum Node<R: Renderer> {
    /// An element node.
    Element(R::Element),
    /// A text node.
    Text(R::Text),
}

/// Convert a renderer-specific node to a generic Node.
impl<R: Renderer> From<R::Element> for Node<R> {
    fn from(el: R::Element) -> Self {
        Node::Element(el)
    }
}

impl<R: Renderer> From<R::Text> for Node<R> {
    fn from(text: R::Text) -> Self {
        Node::Text(text)
    }
}
```

---

## Mount Function

```rust
/// Mount a component into a renderer at the given root.
pub fn mount<C: Component, R: Renderer>(
    component: C,
    renderer: R,
    root: &R::Element,
) -> MountHandle<R> {
    // 1. Create the component
    // 2. Render it to an Element tree
    // 3. Walk the tree, calling renderer methods to create nodes
    // 4. Attach event listeners
    // 5. Set up signal subscriptions for dynamic bindings
    // 6. Return a handle for unmounting
    MountHandle { renderer, root: root.clone() }
}
```

---

## Comparison with Competitors

| Feature | React | Dioxus | Leptos | rye |
|---|---|---|---|---|
| Renderer abstraction | No (DOM only) | Yes (VDOM + patches) | Yes (generic over renderer) | Yes (Renderer trait) |
| Web | Yes | Yes | Yes | Yes (DomRenderer) |
| Native GPU | No | No (WebView only) | No | Yes (NativeRenderer) |
| SSR | Yes | Yes | Yes | Yes (SsrRenderer) |
| Test | Yes (test utils) | No | No | Yes (TestRenderer) |
| Custom renderers | No | No | Yes | Yes |

---

*This document defines the renderer trait. **Implemented** in `rye-core/src/renderer.rs` (`Renderer` trait, `BatchRenderer` trait, `Node<R>` wrapper, `mount()` function), `rye-html/src/dom_renderer.rs` (`DomRenderer`), `rye-desktop` (`NativeRenderer` with wgpu), `rye-ssr/src/render.rs` (`SsrRenderer`), and `rye-testing` (`TestRenderer`).*
