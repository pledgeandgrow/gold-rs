# Template Macro Design

> Goal 13 — `template!` macro that parses HTML-like syntax at compile time, generates optimized static templates + dynamic bindings, supports components/conditionals/loops/fragments, and provides excellent compile errors.

---

## Design Goals

- **HTML-like** — Web developers feel at home
- **Compile-time** — Static parts analyzed and optimized at build time
- **Excellent errors** — Custom diagnostics, not opaque macro panics
- **Components in templates** — Use components like HTML elements
- **Control flow** — Conditionals, loops, fragments native to the macro
- **Type-safe** — Attributes and props validated at compile time

---

## Syntax

### Basic elements

```rust
template! {
    div {
        class: "container",
        id: "main",
        h1 { "Hello, World!" }
        p { "Welcome to rye." }
    }
}
```

### Dynamic content

```rust
let name = Signal::new("Jane");
let count = Signal::new(0);

template! {
    div {
        h1 { "Hello, " {name} "!" }
        p { "You have " {count} " messages." }
    }
}
```

### Attributes with dynamic values

```rust
let is_active = Signal::new(true);
let color = Signal::new("blue");

template! {
    div {
        class: {if is_active() { "active" } else { "inactive" }},
        style: {format!("color: {}", color())},
        "Content"
    }
}
```

### Event handlers

```rust
// Full closure (access event)
button {
    onclick: move |e| {
        log::info!("Clicked at {}, {}", e.client_x(), e.client_y());
        count += 1;
    },
    "Click me"
}

// Shorthand (no event access)
button {
    onclick: count += 1,
    "Increment"
}

// Function call shorthand
button {
    onclick: do_something(),
    "Do something"
}
```

### Conditionals

```rust
let show = Signal::new(true);
let count = Signal::new(5);

template! {
    div {
        if show() {
            p { "Visible!" }
        }

        match count() {
            0 => p { "No items" },
            n if n > 10 => p { "Many items" },
            n => p { {n} " items" },
        }
    }
}
```

### Loops

```rust
let items = Signal::new(vec!["a", "b", "c"]);

template! {
    ul {
        For each(item in items()) {
            key: item,
            li { {item} }
        }
    }
}
```

### Fragments

```rust
template! {
    fragment {
        h1 { "Title" }
        p { "Paragraph" }
        button { "Click" }
    }
}
```

### Components in templates

```rust
template! {
    div {
        class: "app",
        Header { title: "My App" }
        main {
            Counter { initial: 0, step: 1 }
        }
        Footer {}
    }
}
```

### Spread attributes

```rust
let attrs = vec![("data-id", "123"), ("role", "button")];

template! {
    div {
        class: "btn",
        ..attrs,
        "Click"
    }
}
```

---

## Compile-Time Output

The `template!` macro analyzes the template at compile time and generates optimized code:

### Static analysis

```rust
// Input:
template! {
    div {
        class: "container",
        h1 { "Hello, " {name} "!" }
    }
}

// Generated code (conceptual):
{
    // Static parts created once
    static TEMPLATE: Template = Template::new_static(&[
        TemplateNode::Element {
            tag: "div",
            static_attrs: &[("class", "container")],
            children: &[
                TemplateNode::Element {
                    tag: "h1",
                    static_attrs: &[],
                    children: &[
                        TemplateNode::Text("Hello, "),
                        TemplateNode::Dynamic(0),  // {name}
                        TemplateNode::Text("!"),
                    ],
                },
            ],
        },
    ]);

    // Dynamic bindings created at runtime
    let bindings = vec![
        Binding::Text(name.clone()),  // slot 0
    ];

    Element::Template(TEMPLATE, bindings)
}
```

### Optimization: static text concatenation

```rust
// Input: h1 { "Hello, " {name} "!" }
// Optimized: static "Hello, " and "!" are concatenated into the template
// Only {name} is a dynamic binding
```

### Optimization: keyed list reconciliation

```rust
// For each loops, the macro generates keyed reconciliation code:
// - If key exists in previous render → move/reuse node
// - If key is new → create node
// - If key is gone → destroy node
```

---

## Compile-Time Validation

The macro validates at compile time:

| Check | Error if violated |
|---|---|
| Tag names are valid HTML5 tags | "Unknown HTML tag 'xyz'. Did you mean 'div'?" |
| Attribute names are valid | "Invalid attribute name 'data-@foo'" |
| Component names are PascalCase | "Component names must be PascalCase, got 'counter'" |
| Required props are provided | "Component 'Button' requires prop 'label'" |
| Prop types match | "Prop 'size' expects i32, got String" |
| Event handler is a closure or shorthand | "onclick must be a closure or signal expression" |
| `key:` is present in `For` loops | "For loops require a `key:` attribute for reconciliation" |

---

## Error Messages

### Bad tag name

```rust
template! {
    dv { "Hello" }
}
```

```
error: Unknown HTML tag 'dv'
  --> src/app.rs:12:5
   |
12 |     dv { "Hello" }
   |     ^^
   |
   = hint: Did you mean 'div'?
   = help: Valid HTML tags: div, span, p, h1-h6, ul, li, button, ...
```

### Missing required prop

```rust
template! {
    Button { disabled: true }
}
```

```
error: Missing required prop 'label' for component 'Button'
  --> src/app.rs:15:5
   |
15 |     Button { disabled: true }
   |     ^^^^^^
   |
   = note: Component 'Button' requires:
   |         label: String  (required)
   |         disabled: bool (optional)
   |
   = help: Add the missing prop:
   |         Button { label: "Click me", disabled: true }
```

### Wrong prop type

```rust
template! {
    Button { label: 42 }
}
```

```
error: Prop 'label' expects String, got integer
  --> src/app.rs:15:20
   |
15 |     Button { label: 42 }
   |                    ^^
   |
   = help: Use a string literal:
   |         Button { label: "42".to_string() }
```

---

## Comparison with Competitors

| Feature | Dioxus RSX | Leptos view! | Yew html! | rye template! |
|---|---|---|---|---|
| Syntax | RSX (JSX-like) | HTML-like | JSX-like | HTML-like |
| Compile-time parsing | Yes | Yes | Yes | Yes |
| Custom errors | No (macro panics) | No | No | Yes (custom diagnostics) |
| Static optimization | Partial | Yes | No | Yes |
| Keyed lists | Manual | Built-in | Manual | Built-in |
| Component props typed | Yes | Yes | No (html!) | Yes |
| Shorthand event handlers | No | No | No | Yes |
| Implicit template! | No | No | No | Yes (in #[component]) |

---

*This document defines the template macro design. **Implemented** in `rye-macros/src/template.rs` — compile-time parsing, static/dynamic separation, keyed list reconciliation, component prop validation, spread attributes, and custom error diagnostics.*
