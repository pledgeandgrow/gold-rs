# Context & Dependency Injection Design

> Goal 18 — Built-in context system (like React Context but type-safe and reactive). No prop drilling, no external state library needed for most cases.

---

## Design Goals

- **Type-safe** — Context values are typed, no `dyn Any` casting
- **Reactive** — Context values can be signals, and changes propagate
- **Zero prop drilling** — Provide at top, consume anywhere below
- **No external library** — Built into core, not a separate crate
- **Scoped** — Each component subtree can have its own context provider

---

## API

### Provide context

```rust
/// Provide a value to all descendant components.
/// The value is available via `use_context::<T>()` in any child.
pub fn provide_context<T: 'static>(value: T);

/// Provide a reactive signal as context.
/// Consumers get a Signal<T> that updates reactively.
pub fn provide_context_signal<T: 'static>(signal: Signal<T>);
```

### Consume context

```rust
/// Get a context value of type T from the nearest ancestor provider.
/// Panics if no provider exists (use try_context for non-panicking version).
pub fn use_context<T: 'static>() -> T;

/// Try to get a context value. Returns None if no provider exists.
pub fn try_context<T: 'static>() -> Option<T>;

/// Get a reactive signal from context.
pub fn use_context_signal<T: Clone + 'static>() -> Signal<T>;
```

---

## Usage

### Theme context

```rust
// Define theme type
#[derive(Clone)]
struct Theme {
    primary: String,
    background: String,
    text: String,
}

// Provide at app root
#[component]
fn App() {
    let theme = use_signal(|| Theme {
        primary: "#007bff".to_string(),
        background: "#ffffff".to_string(),
        text: "#333333".to_string(),
    });

    provide_context_signal(theme);

    div {
        class: "app",
        Header {}
        Content {}
        Footer {}
    }
}

// Consume in any child component
#[component]
fn Header() {
    let theme = use_context_signal::<Theme>();

    header {
        style: {format!("background: {}; color: {}", theme().background, theme().text)},
        h1 { "My App" }
    }
}
```

### Auth context

```rust
#[derive(Clone)]
struct AuthState {
    user: Option<User>,
    token: Option<String>,
}

#[component]
fn App() {
    let auth = use_signal(|| AuthState {
        user: None,
        token: None,
    });

    provide_context_signal(auth);

    // Route guards can access auth
    Router {
        Route { path: "/", Home {} }
        Route { path: "/login", Login {} }
        Route {
            path: "/dashboard",
            guard: move || use_context_signal::<AuthState>().user.is_some(),
            Dashboard {}
        }
    }
}

#[component]
fn Dashboard() {
    let auth = use_context_signal::<AuthState>();

    if let Some(user) = &auth().user {
        div { "Welcome, " {user.name} }
    } else {
        // This shouldn't happen if guard works, but type-safe
        Redirect { to: "/login" }
    }
}
```

### Multiple contexts

```rust
#[component]
fn App() {
    provide_context_signal(use_signal(|| Theme::default()));
    provide_context_signal(use_signal(|| AuthState::default()));
    provide_context_signal(use_signal(|| Locale::default()));
    provide_context(use_signal(|| ApiClient::new()));

    // All available to any descendant
    Main {}
}

// Consume multiple
#[component]
fn Profile() {
    let theme = use_context_signal::<Theme>();
    let auth = use_context_signal::<AuthState>();
    let locale = use_context_signal::<Locale>();
    let api = use_context::<ApiClient>();
    // ...
}
```

---

## How It Works

Each component has a **context map** — a `HashMap<TypeId, Box<dyn Any>>`. When a component provides context, it stores the value in its context map. When a child consumes context, it walks up the component tree until it finds a provider.

```rust
/// Context map for a component scope.
pub struct ContextMap {
    entries: HashMap<TypeId, Box<dyn Any>>,
}

impl ContextMap {
    fn provide<T: 'static>(&mut self, value: T) {
        self.entries.insert(TypeId::of::<T>(), Box::new(value));
    }

    fn get<T: 'static>(&self) -> Option<&T> {
        self.entries
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
    }
}
```

### Context resolution

```
Component A (provides Theme)
└─ Component B
   └─ Component C (consumes Theme)
      └─ Component D

Resolution for use_context::<Theme>() in C:
  1. Check C's context map → not found
  2. Check B's context map → not found
  3. Check A's context map → found! Return &Theme
```

### Override context

A child can override a parent's context:

```rust
#[component]
fn App() {
    provide_context_signal(use_signal(|| Theme::light()));

    div {
        // This subtree uses dark theme
        div {
            provide_context_signal(use_signal(|| Theme::dark())),
            Sidebar {}  // gets dark theme
        }
        Main {}  // gets light theme (from App)
    }
}
```

---

## Comparison with Competitors

| Feature | React | Vue | Dioxus | Leptos | rye |
|---|---|---|---|---|---|
| Context API | Yes (untyped) | provide/inject | use_context | use_context | use_context |
| Type-safe | No (TS only) | No (TS only) | Yes | Yes | Yes |
| Reactive | Yes (if state changes) | Yes | Yes | Yes | Yes (Signal<T>) |
| Override in subtree | Yes | Yes | Yes | Yes | Yes |
| Multiple contexts | Yes | Yes | Yes | Yes | Yes |
| External library needed | Often (Redux, Zustand) | Often (Pinia) | No | No | No |

---

*This document defines the context/DI system. **Implemented** in `rye-core/src/context.rs` — `ContextMap` with `TypeId`-keyed lookup, `provide_context`, `provide_context_signal`, `use_context`, `use_context_signal`, hierarchical resolution with subtree override.*
