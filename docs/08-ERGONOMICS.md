# Developer Ergonomics — Reducing Boilerplate & Friction

> Goal: Make `rye` the most ergonomic Rust UI framework. Minimize ceremony for new developers while preserving Rust's safety guarantees.

---

## Decisions

### 1. Implicit `template!` in `#[component]` (P0)

The `#[component]` macro auto-wraps the function body in `template!{}`. No need to manually write `template!`.

**Before:**
```rust
#[component]
fn Counter() -> Element {
    let count = use_signal(|| 0);

    template! {
        div {
            h1 { "Count: " {count} }
            button { onclick: move |_| count += 1, "Increment" }
        }
    }
}
```

**After:**
```rust
#[component]
fn Counter() {
    let count = use_signal(|| 0);

    div {
        h1 { "Count: " {count} }
        button { onclick: count += 1, "Increment" }
    }
}
```

**How it works:**
- The `#[component]` macro parses the function body
- If the last expression is a block containing HTML-like elements (`div`, `h1`, `span`, etc.), it wraps it in `template!{}`
- If the last expression is already `template!{}`, it leaves it as-is
- If the last expression is something else (e.g., a match, an if/else), the macro attempts to wrap each branch
- If no template-like structure is found, compile error with helpful message

**Edge cases:**
```rust
// Conditional rendering — both branches auto-wrapped
#[component]
fn Greeting(name: String) {
    if name.is_empty() {
        p { "Hello, stranger!" }
    } else {
        p { "Hello, " {name} "!" }
    }
}

// Match — all arms auto-wrapped
#[component]
fn Status(state: State) {
    match state {
        State::Loading => div { "Loading..." },
        State::Ready(data) => div { {data} },
        State::Error(err) => div { class: "error", {err} },
    }
}

// Explicit template! still works (for advanced cases)
#[component]
fn Complex() -> Element {
    let v = compute();
    template! {
        div { {v} }
    }
}
```

---

### 2. `onclick: count += 1` Shorthand (P0)

Eliminate `move |_|` for simple event handlers.

**Before:**
```rust
button { onclick: move |_| count += 1, "Increment" }
button { onclick: move |_| count -= 1, "Decrement" }
button { onclick: move |_| count.set(0), "Reset" }
```

**After:**
```rust
button { onclick: count += 1, "Increment" }
button { onclick: count -= 1, "Decrement" }
button { onclick: count.set(0), "Reset" }
```

**How it works:**
- The `template!` macro detects event handler values (`onclick`, `oninput`, `onchange`, etc.)
- If the value is a simple expression (not a closure), it auto-wraps in `move |_| { ... }`
- Supported shorthand patterns:
  - `onclick: count += 1` → `onclick: move |_| count += 1`
  - `onclick: count -= 1` → `onclick: move |_| count -= 1`
  - `onclick: count.set(0)` → `onclick: move |_| count.set(0)`
  - `onclick: count.update(|v| v + 1)` → `onclick: move |_| count.update(|v| v + 1)`
  - `onclick: do_something()` → `onclick: move |_| do_something()`
- If the value is already a closure (`move |_| ...` or `|e| ...`), it's left as-is
- If the value is a variable holding a closure, it's left as-is

**When you still need `move |_|`:**
```rust
// Accessing the event object
button { onclick: move |e| { log(&e); count += 1; }, "Click" }

// Multiple statements
button { onclick: move |_| {
    count += 1;
    log_count();
    save_to_db();
}, "Save" }

// Conditional logic
button { onclick: move |_| {
    if count() > 10 {
        count.set(0);
    } else {
        count += 1;
    }
}, "Smart Increment" }
```

---

### 3. `use rye::prelude::*;` (P0)

One import line brings in everything needed for most components.

**What the prelude includes:**

```rust
// Re-exported from rye::prelude
pub use rye_core::{Component, Element, Renderer};
pub use rye_signals::{Signal, Memo, Effect, Resource, ResourceState, GlobalSignal, batch};
pub use rye_macros::{template, component};

// Hooks
pub use rye_signals::{use_signal, use_memo, use_effect, use_resource, use_ref, use_context};

// Control flow components
pub use rye_core::{Show, For, Suspense, ErrorBoundary, Fragment, KeepAlive};

// Router (if rye-router is in dependencies)
pub use rye_router::{Router, Route, Link};

// HTML elements (auto-imported by template! macro, not explicit)
// div, span, h1-h6, p, button, input, form, ul, li, etc.

// Event types
pub use rye_core::{ClickEvent, InputEvent, KeyEvent, FocusEvent};
```

**Usage:**
```rust
use rye::prelude::*;

#[component]
fn Counter() {
    let count = use_signal(|| 0);
    div {
        h1 { "Count: " {count} }
        button { onclick: count += 1, "Increment" }
    }
}
```

**What's NOT in the prelude (must be imported explicitly):**
- Forms (`Form`, `Field`, `ValidationRule`) — from `rye::forms`
- i18n (`Locale`, `MessageStore`) — from `rye::i18n`
- Animations (`Transition`, `Spring`) — from `rye::animations`
- Devtools (`Inspector`, `Profiler`) — from `rye::devtools`
- Testing utilities — from `rye::testing`
- SSR functions — from `rye::ssr`

This keeps the prelude lean while covering 95% of component code.

---

### 4. Auto-Return Type Inference (P0)

Drop `-> Element` from component signatures.

**Before:**
```rust
#[component]
fn Counter() -> Element {
    let count = use_signal(|| 0);
    template! { div { {count} } }
}
```

**After:**
```rust
#[component]
fn Counter() {
    let count = use_signal(|| 0);
    div { {count} }
}
```

**How it works:**
- The `#[component]` macro adds `-> Element` automatically
- If the developer explicitly writes `-> Element`, it's preserved (no conflict)
- If the developer writes a different return type, compile error: "Components must return Element"

---

### 5. `rpg add component` CLI (P1)

Scaffold new components from the command line.

**Usage:**
```bash
rpg add component Counter
rpg add component UserCard --path src/components/user_card.rs
rpg add component Dialog --with-props
rpg add component DataTable --with-props --with-style
```

**Generated file (`src/components/counter.rs`):**
```rust
use rye::prelude::*;

#[component]
fn Counter() {
    // TODO: implement your component
    div {
        "Counter"
    }
}
```

**With `--with-props` (`src/components/dialog.rs`):**
```rust
use rye::prelude::*;

#[derive(Props)]
struct DialogProps {
    title: String,
    #[prop(optional)]
    open: bool,
}

#[component]
fn Dialog(props: DialogProps) {
    if props.open {
        div {
            class: "dialog",
            h2 { {props.title} }
        }
    }
}
```

**With `--with-style` (`src/components/data_table.rs`):**
```rust
use rye::prelude::*;

#[derive(Props)]
struct DataTableProps {
    rows: Vec<Row>,
}

style! {
    .data-table {
        width: 100%;
        border-collapse: collapse;
    }
    .data-table th {
        text-align: left;
        padding: 8px;
    }
}

#[component]
fn DataTable(props: DataTableProps) {
    table {
        class: "data-table",
        For each(row in props.rows) {
            tr {
                key: row.id,
                td { {row.name} }
            }
        }
    }
}
```

**CLI auto-registers the component:**
- Adds `mod counter;` to `src/components/mod.rs` (or creates it)
- If no `mod.rs` exists, adds `mod components;` to `main.rs` / `lib.rs`

---

### 6. `.rpg` Single-File Format (P2)

A Vue/Svelte-like single-file component format for developers who prefer co-located template, logic, and styles.

**File: `src/components/counter.rpg`**
```rpg
<script>
let count = use_signal(|| 0);
</script>

<template>
<div class="counter">
    <h1>Count: {count}</h1>
    <button onclick="count += 1">Increment</button>
    <button onclick="count -= 1">Decrement</button>
</div>
</template>

<style scoped>
.counter {
    padding: 24px;
    text-align: center;
}
</style>
```

**Build step:**
The CLI's build process transforms `.rpg` files into `.rs` files before compilation:

```rust
// Generated from counter.rpg (not written by developer)
use rye::prelude::*;

style! {
    .counter {
        padding: 24px;
        text-align: center;
    }
}

#[component]
fn Counter() {
    let count = use_signal(|| 0);

    div {
        class: "counter",
        h1 { "Count: " {count} }
        button { onclick: count += 1, "Increment" }
        button { onclick: count -= 1, "Decrement" }
    }
}
```

**Syntax mapping:**

| `.rpg` syntax | Generated Rust |
|---|---|
| `<template>...</template>` | `template! { ... }` (or implicit in `#[component]`) |
| `<div class="x">` | `div { class: "x", }` |
| `<button onclick="count += 1">` | `button { onclick: count += 1, }` |
| `{count}` | `{count}` (same) |
| `<script>...</script>` | Rust code in the component function body |
| `<style scoped>...</style>` | `style! { ... }` with scoped class hashing |
| `<style>...</style>` | `style! { ... }` global (no scoping) |

**File naming convention:**
- `src/components/counter.rpg` → generates `src/components/counter.rs`
- Component name derived from filename: `counter` → `Counter`
- The CLI watches `.rpg` files and regenerates `.rs` on change during `rpg dev`

**When to use `.rpg` vs `.rs`:**
- `.rpg` — beginners, simple components, co-located styles
- `.rs` — advanced components, complex logic, full Rust IDE support
- Both can coexist in the same project
- The CLI handles both transparently

**Trade-offs:**

| Aspect | `.rpg` files | `.rs` files |
|---|---|---|
| Learning curve | Lower (familiar to Vue/Svelte devs) | Higher (pure Rust) |
| IDE support | Limited (needs custom LSP) | Full (rust-analyzer) |
| Type checking | Via generated code (errors map back) | Direct |
| Hot reload | Template + style hot reload | Template hot reload |
| Debugging | Through generated code | Direct |
| Flexibility | Constrained to SFC structure | Full Rust flexibility |

**Implementation plan:**
- P2 (post-V1 MVP) — defer until the pure-Rust experience is solid
- Requires: `.rpg` parser, code generator, file watcher integration, LSP for `.rpg` files
- The generated `.rs` files are committed to `.gitignore` (build artifacts)

---

## Summary: Simplest Possible Component

With all P0 ergonomics applied:

```rust
use rye::prelude::*;

#[component]
fn HelloWorld() {
    h1 { "Hello, World!" }
}
```

**4 lines.** Compare:
- React: `function HelloWorld() { return <h1>Hello, World!</h1> }` — 1 line (but in JS)
- Vue SFC: `<template><h1>Hello</h1></template>` — 1 line (but in .vue file)
- Svelte: `<h1>Hello</h1>` — 1 line (but in .svelte file)
- Dioxus: `fn HelloWorld() -> Element { rsx! { h1 { "Hello" } } }` — 5 lines
- Yew: `fn HelloWorld() -> Html { html! { <h1>{"Hello"}</h1> } }` — 5 lines

With `.rpg` files (P2):
```rpg
<template>
<h1>Hello, World!</h1>
</template>
```

**3 lines.** Competitive with Vue/Svelte.

---

## CLI Command Reference

```bash
# Create new project
rpg new my-app --template web
rpg new my-app --template desktop
rpg new my-app --template fullstack

# Development
rpg dev                    # start dev server with hot reload
rpg dev --port 3000        # custom port

# Build
rpg build --target web     # WASM for web
rpg build --target desktop # native desktop binary
rpg build --target ssr     # SSR binary

# Testing
rpg test                   # run all tests
rpg test --unit            # unit tests only
rpg test --e2e             # E2E tests

# Component scaffolding
rpg add component Counter
rpg add component Dialog --with-props
rpg add component Card --with-props --with-style

# Deployment
rpg deploy --target web    # deploy to static host
rpg deploy --target desktop # package installer

# Maintenance
rpg upgrade                # upgrade rye with codemods
rpg add @rye/ui            # add component library from registry
```

---

*This document defines the developer ergonomics strategy. **Implemented** — P0 items (implicit `template!`, event shorthand, prelude, return type inference) in `rye-macros/src/component.rs` and `rye-macros/src/template.rs`. P1 items (`rpg add component` CLI) in `rye-cli`. P2 (`.rpg` single-file format) is deferred post-V1 as designed.*
