# Template Syntax — `template!` Macro

> Goal 4: Choose and document the template syntax for `rye`.

---

## Decision: HTML-Like Macro Syntax with Rust Expressions

We adopt an **HTML-like macro syntax** that:
- Feels familiar to web developers (React JSX, Vue templates, Svelte markup)
- Supports full Rust expressions in dynamic parts
- Compiles to optimized static templates + dynamic bindings at build time
- Produces excellent compile errors (not opaque macro panics)

---

## Syntax Overview

### Basic Structure

```rust
template! {
    div {
        class: "container",
        h1 { "Hello, World!" }
        p { "Welcome to rye." }
    }
}
```

### Dynamic Content

Curly braces `{expression}` denote dynamic Rust expressions:

```rust
template! {
    div {
        h1 { "Hello, " {name} "!" }
        p { "You have " {count} " messages." }
        span { format!("Score: {:.2}", score) }
    }
}
```

### Attributes

```rust
template! {
    button {
        class: "btn",
        class: if is_active { "btn-active" },     // conditional class
        disabled: !is_enabled,                      // boolean attribute
        onclick: move |_| submit(),                 // event handler
        "Submit"
    }
}
```

### Conditional Rendering

```rust
template! {
    div {
        if user.is_admin() {
            span { class: "badge", "Admin" }
        } else if user.is_mod() {
            span { class: "badge", "Moderator" }
        } else {
            span { class: "badge", "User" }
        }
    }
}
```

### List Rendering

```rust
template! {
    ul {
        For each(item in items) {
            li {
                key: item.id,                       // keyed for efficient reconciliation
                class: if item.done { "completed" },
                {item.label()}
                button { onclick: move |_| remove(item.id), "Delete" }
            }
        }
    }
}
```

### Components

```rust
template! {
    div {
        Card {
            title: "Settings",
            Card::Body {
                p { "Configure your preferences." }
            }
        }
    }
}
```

### Slots / Named Children

```rust
template! {
    Dialog {
        Dialog::Header { "Confirm Action" }
        Dialog::Body {
            p { "Are you sure you want to continue?" }
        }
        Dialog::Footer {
            button { onclick: move |_| cancel(), "Cancel" }
            button { onclick: move |_| confirm(), "Confirm" }
        }
    }
}
```

### Fragments

```rust
template! {
    Fragment {
        h1 { "Title" }
        p { "Subtitle" }
    }
}
```

### Spread Attributes

```rust
template! {
    div {
        ..props,                    // spread all props as attributes
        class: "extra-class",       // explicit attrs override spread
    }
}
```

### Reactive Attributes (Signal-Bound)

```rust
template! {
    div {
        class: move || format!("card {}", if *theme.read() == "dark" { "dark" } else { "light" }),
        style: { color: text_color() },
    }
}
```

### Event Modifiers

```rust
template! {
    div {
        onclick: move |_| handle_click(),
        onclick:prevent_default: move |_| handle_submit(),  // preventDefault
        onclick:stop_propagation: move |_| handle_inner(),   // stopPropagation
        onkeydown:once: move |_| handle_first_press(),       // fire once
        oninput:debounce(300): move |_| handle_search(),     // debounced
    }
}
```

### Style Binding

```rust
template! {
    div {
        style: {
            color: "red",
            background: if is_dark { "#333" } else { "#fff" },
            transform: format!("translateX({}px)", offset()),
        }
    }
}
```

---

## Design Rationale

### Why HTML-Like, Not JSX-Like?

| Criterion | HTML-like (`template!`) | JSX-like (`rsx!`) |
|---|---|---|
| Familiarity to web devs | High (Vue, Svelte, HTML) | Medium (React only) |
| Rust integration | Expressions in `{}` | Expressions in `{}` |
| Attribute syntax | `key: value` (Rust-like) | `key={value}` (JS-like) |
| Compile errors | Easier to produce good errors | Harder (JSX parsing in Rust) |
| Nesting | Indentation-based | Explicit closing tags |
| Tooling | Simpler to parse | More complex parser |

We chose HTML-like because:
1. **Broader appeal** — Vue, Svelte, Angular, plain HTML devs all find it familiar
2. **Rust-native attribute syntax** — `class: "btn"` looks like Rust struct fields
3. **Better error messages** — simpler grammar = better diagnostics
4. **Tree-sitter friendly** — easier to write a grammar for IDE support

### Why Not a Separate DSL (like Svelte)?

A separate DSL would require:
- A custom compiler (not just a proc-macro)
- Separate file format (`.rye` files)
- Separate tooling (formatter, linter, language server)
- More friction for Rust developers

By using a Rust proc-macro, we get:
- Full Rust compiler support (type checking, borrow checker)
- `cargo fmt` compatibility (with custom macro formatting)
- `rust-analyzer` integration
- No new file format — it's just Rust

### Why Not Just Use `format!` Strings?

String-based templating (like `format!("<div>{}</div>", value)`) loses:
- Compile-time validation
- Type safety
- Static/dynamic separation for optimization
- XSS protection

The `template!` macro gives us all of these for free.

---

## Compile-Time Behavior

The macro performs these steps at compile time:

1. **Parse** the template into an AST (elements, attributes, text, expressions)
2. **Type-check** all expressions against the component's scope
3. **Validate** attribute names, event names, component props
4. **Separate** static parts from dynamic bindings
5. **Generate** Rust code that:
   - Creates a `Template` struct for static parts (created once, reused)
   - Wires dynamic bindings to signal subscriptions
   - Produces a `Element` return value

### Generated Code (Conceptual)

Input:
```rust
template! {
    div {
        class: "card",
        h1 { {title()} }
    }
}
```

Conceptual output:
```rust
{
    static TEMPLATE: Template = Template::new_static([
        Element("div", [Attr("class", "card")], [
            Element("h1", [], [Dynamic(0)])
        ])
    ]);

    Element::from_template(&TEMPLATE, |ctx| {
        ctx.bind_text(0, move || title());
    })
}
```

---

## Error Messages

The macro includes a custom diagnostic layer:

```
error[rye-001]: Unknown attribute `colr` — did you mean `color`?
  --> src/app.rs:12:5
   |
12 |     div { colr: "red" }
   |          ^^^^
   |
   = help: Valid style attributes include: color, background, font-size, ...
   = note: See the styling guide: https://rye.rs/docs/styling
```

```
error[rye-002]: Component `Card` requires prop `title` but it was not provided.
  --> src/app.rs:25:5
   |
25 |     Card { body: "Hello" }
   |     ^^^^
   |
   = help: Add `title: "..."` to the Card component
   = note: Required props: title. Optional props: body, footer.
```

---

## Comparison with Competitors

| Feature | React JSX | Vue SFC | Svelte | Dioxus RSX | `rye template!` |
|---|---|---|---|---|---|
| Language | JS/TS extension | Custom DSL | Custom DSL | Rust macro | Rust macro |
| Compile-time validation | TS types (if used) | Runtime + Volar | Compile-time | Compile-time | Compile-time (full) |
| Static/dynamic separation | No | Partial | Yes | Yes | Yes |
| Error messages | TS errors | Volar warnings | Good | Opaque macro errors | Custom diagnostics |
| Full language expressions | Yes (JS) | Limited | Limited | Yes (Rust) | Yes (Rust) |
| Event modifiers | No | Yes (`.prevent`) | Yes (`|preventDefault`) | No | Yes (`:prevent_default`) |
| Keyed lists | `key={}` | `:key` | `|key` | `key:` | `key:` |
| Slots/named children | Children prop | Named slots | Slots | Children prop | Named slots |

---

*This document defines the template syntax. **Implemented** in `rye-macros/src/template.rs` with compile-time parsing, validation, static optimization, and custom error diagnostics.*
