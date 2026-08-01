# Styling System Design

> Goal 20 — Multi-approach styling: scoped CSS in components, CSS-in-Rust (typed, compile-time validated), Tailwind class support (built-in), CSS variables with reactive bindings, style co-location in component files.

---

## Design Goals

- **Choice** — Multiple styling approaches, use what fits
- **Compile-time** — CSS validated at compile time where possible
- **Scoped by default** — Component styles don't leak
- **Reactive** — CSS variables can be bound to signals
- **Tailwind built-in** — No config needed, tree-shaken
- **Co-located** — Styles live next to components

---

## Four Styling Approaches

### 1. Scoped CSS (`style!` macro)

```rust
use rye::prelude::*;

style! {
    .counter {
        padding: 24px;
        text-align: center;

        h1 {
            font-size: 2rem;
            color: #333;
        }

        button {
            padding: 8px 16px;
            margin: 4px;
            border: none;
            border-radius: 4px;
            background: #007bff;
            color: white;
            cursor: pointer;

            &:hover {
                background: #0056b3;
            }
        }
    }
}

#[component]
fn Counter() {
    let count = use_signal(|| 0);

    div {
        class: "counter",  // Scoped class — auto-hashed
        h1 { "Count: " {count} }
        button { onclick: count += 1, "+" }
    }
}
```

**How scoping works:**
- The `style!` macro hashes class names: `.counter` → `.counter_a3f2b1`
- The same hash is applied to `class: "counter"` in the template
- Styles never leak to other components
- Dev tools show the original class name (source maps)

### 2. CSS-in-Rust (typed, compile-time validated)

```rust
use rye::prelude::*;

#[component]
fn Button(props: ButtonProps) {
    let styles = css! {
        padding: Px(8, 16),
        border: None,
        border_radius: Px(4),
        background: if props.disabled {
            Color::Gray
        } else {
            Color::Hex("#007bff")
        },
        color: Color::White,
        cursor: if props.disabled { Cursor::NotAllowed } else { Cursor::Pointer },
    };

    button {
        style: styles,
        disabled: props.disabled,
        {props.label}
    }
}
```

**Type-safe CSS:**
- Every CSS property is a typed Rust value
- Invalid values are compile errors
- No string typos in CSS
- Autocomplete in IDE

### 3. Tailwind (built-in, no config)

```rust
use rye::prelude::*;

#[component]
fn Card(props: CardProps) {
    div {
        class: "bg-white rounded-lg shadow-md p-6 mb-4",
        h2 {
            class: "text-xl font-bold text-gray-800 mb-2",
            {props.title}
        }
        p {
            class: "text-gray-600",
            {props.body}
        }
    }
}
```

**How it works:**
- rye includes a built-in Tailwind CSS compiler
- Only used classes are included in the final CSS (tree-shaking)
- No `tailwind.config.js` needed — works out of the box
- Custom config via `rye.toml` for theme extension
- JIT compilation — classes generated on demand

### 4. CSS Variables with reactive bindings

```rust
use rye::prelude::*;

#[component]
fn ThemeToggle() {
    let dark_mode = use_signal(|| false);

    div {
        // Reactive CSS variable — updates when signal changes
        style: {
            "--bg-color": if dark_mode() { "#1a1a1a" } else { "#ffffff" },
            "--text-color": if dark_mode() { "#ffffff" } else { "#333333" },
        },
        class: "app-container",

        button {
            onclick: dark_mode = !dark_mode(),
            "Toggle theme"
        }
    }
}
```

**In CSS:**
```css
.app-container {
    background: var(--bg-color);
    color: var(--text-color);
    transition: background 0.3s, color 0.3s;
}
```

---

## Style Co-location

### In `.rs` files

```rust
// counter.rs
use rye::prelude::*;

style! {
    .counter {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
    }
}

#[component]
fn Counter() {
    let count = use_signal(|| 0);
    div {
        class: "counter",
        h1 { {count} }
        button { onclick: count += 1, "+" }
    }
}
```

### In `.rpg` files (P2)

```rpg
<!-- counter.rpg -->
<script>
let count = use_signal(|| 0);
</script>

<template>
<div class="counter">
    <h1>{count}</h1>
    <button onclick="count += 1">+</button>
</div>
</template>

<style scoped>
.counter {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
}
</style>
```

---

## Dynamic Styles

### Conditional classes

```rust
let is_active = use_signal(|| true);

div {
    class: {
        if is_active() { "btn btn-active" }
        else { "btn btn-inactive" }
    },
    "Button"
}
```

### Class lists

```rust
let is_primary = use_signal(|| true);
let is_large = use_signal(|| false);

div {
    class: class_list!(
        "btn",
        "btn-primary" if is_primary(),
        "btn-large" if is_large(),
        "btn-disabled" if false,
    ),
    "Button"
}
```

### Inline styles with signals

```rust
let width = use_signal(|| 100);

div {
    style: {format!("width: {}px; transition: width 0.3s", width())},
    "Resizable"
}
```

---

## Comparison with Competitors

| Feature | React | Vue | Dioxus | Leptos | rye |
|---|---|---|---|---|---|
| Scoped CSS | CSS Modules | SFC scoped | No | No | Yes (style!) |
| CSS-in-JS | styled-components | No | No | No | Yes (css!) |
| Tailwind | Via plugin | Via plugin | No | No | Built-in |
| CSS variables | Manual | Manual | No | No | Reactive bindings |
| Type-safe CSS | No | No | No | No | Yes (css!) |
| Co-located | No | Yes (SFC) | No | No | Yes |

---

*This document defines the styling system. **Implemented** — `style!` macro (scoped CSS with class hashing), `css!` macro (typed CSS-in-Rust), built-in Tailwind compiler (tree-shaken, JIT), reactive CSS variable bindings, `class_list!` macro, and style co-location. Distributed across `rye-macros` and `rye-core`.*
