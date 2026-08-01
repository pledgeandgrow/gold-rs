# Event System Design

> Goal 15 — Typed events with Rust enums, event delegation, synthetic events for cross-platform consistency, custom user events.

---

## Design Goals

- **Typed** — Events are Rust enums, not `dyn Any`
- **Cross-platform** — Same event types on web, desktop, mobile, and test
- **Delegated** — Event delegation at root (like SolidJS) for performance
- **Custom** — User-defined events for component communication
- **Preventable** — `prevent_default()` and `stop_propagation()` support

---

## Event Types

### Core UI Events

```rust
/// Mouse events — cross-platform (web + native).
#[derive(Debug, Clone)]
pub enum MouseEvent {
    /// Mouse click.
    Click {
        x: f64,
        y: f64,
        button: MouseButton,
        modifiers: Modifiers,
    },
    /// Mouse movement.
    MouseMove { x: f64, y: f64 },
    /// Mouse enters element bounds.
    MouseEnter { x: f64, y: f64 },
    /// Mouse leaves element bounds.
    MouseLeave { x: f64, y: f64 },
    /// Mouse button pressed.
    MouseDown { x: f64, y: f64, button: MouseButton },
    /// Mouse button released.
    MouseUp { x: f64, y: f64, button: MouseButton },
}

/// Keyboard events.
#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    /// Key pressed.
    KeyDown { key: Key, modifiers: Modifiers, repeat: bool },
    /// Key released.
    KeyUp { key: Key, modifiers: Modifiers },
    /// Character input.
    KeyPress { char: char, modifiers: Modifiers },
}

/// Input/form events.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Value changed (fires on every keystroke).
    Input { value: String },
    /// Value committed (fires on blur/enter).
    Change { value: String },
    /// Form submitted.
    Submit { data: FormData },
}

/// Focus events.
#[derive(Debug, Clone)]
pub enum FocusEvent {
    Focus,
    Blur,
}

/// Touch events (mobile + touch screens).
#[derive(Debug, Clone)]
pub enum TouchEvent {
    TouchStart { touches: Vec<Touch> },
    TouchMove { touches: Vec<Touch> },
    TouchEnd { touches: Vec<Touch> },
}

/// Scroll/wheel events.
#[derive(Debug, Clone)]
pub enum ScrollEvent {
    Scroll { dx: f64, dy: f64 },
}

/// Window events.
#[derive(Debug, Clone)]
pub enum WindowEvent {
    Resize { width: u32, height: u32 },
}
```

### Helper types

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,  // Cmd on macOS, Win key on Windows
}

#[derive(Debug, Clone)]
pub struct Touch {
    pub id: u32,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub enum Key {
    Enter,
    Escape,
    Tab,
    Space,
    Backspace,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Char(char),
    Function(u8),  // F1-F12
}
```

---

## Event Handler Signature

```rust
/// Event handlers receive a typed event and can prevent default behavior.
pub trait EventExt {
    /// Prevent the default browser/platform behavior.
    fn prevent_default(&self);

    /// Stop the event from bubbling to parent elements.
    fn stop_propagation(&self);

    /// Check if default was prevented.
    fn is_default_prevented(&self) -> bool;

    /// Check if propagation was stopped.
    fn is_propagation_stopped(&self) -> bool;
}
```

### Usage in templates

```rust
// Typed event — handler receives MouseEvent
button {
    onclick: move |e: MouseEvent| {
        log::info!("Clicked at ({}, {})", e.x(), e.y());
        e.prevent_default();
        count += 1;
    },
    "Click"
}

// Shorthand (no event access needed)
button {
    onclick: count += 1,
    "Increment"
}

// Keyboard handling
input {
    onkeydown: move |e: KeyboardEvent| {
        if e.key == Key::Enter {
            submit();
        }
    }
}
```

---

## Event Delegation

Instead of attaching one event listener per element, rye attaches **one listener per event type at the root**. Events bubble up to the root, and rye dispatches to the correct handler based on the target element.

```
Traditional (React <17):
┌──────────────────────────────────┐
│ div [onclick=handler1]           │
│  └─ button [onclick=handler2]    │  ← 2 listeners
│      └─ span [onclick=handler3]  │  ← 3 listeners
└──────────────────────────────────┘

Delegated (rye, SolidJS):
┌──────────────────────────────────┐
│ #root [onclick=dispatcher]       │  ← 1 listener
│  └─ div                          │
│      └─ button                   │  ← handler stored in map
│          └─ span                 │  ← handler stored in map
└──────────────────────────────────┘

Event flow:
  User clicks span → browser fires click → bubbles to root →
  dispatcher looks up span's handler → calls handler2 →
  if not stopped, calls handler1
```

**Benefits:**
- Fewer DOM/WASM bridge calls (one listener vs N)
- No listener cleanup on element removal (just remove from map)
- Consistent event ordering

**Implementation:**
```rust
/// Event handler registry — maps element IDs to handlers.
pub struct HandlerRegistry {
    handlers: HashMap<(ElementId, &'static str), EventHandler>,
}

/// The root dispatcher — one per event type.
fn dispatch_event(event_type: &str, event: &dyn Any) {
    let target = get_event_target(event);
    let element_id = get_element_id(target);

    // Bubble up through ancestors
    let mut current = Some(element_id);
    while let Some(id) = current {
        if let Some(handler) = registry.get(&(id, event_type)) {
            handler(event);
            if event.is_propagation_stopped() {
                break;
            }
        }
        current = get_parent(id);
    }
}
```

---

## Custom Events

Components can emit custom events for parent communication:

```rust
// Define a custom event
#[derive(Debug, Clone)]
pub struct SearchEvent {
    pub query: String,
    pub filters: Vec<String>,
}

// Component emits event
#[component]
fn SearchBar(on_search: impl Fn(SearchEvent)) {
    let query = use_signal(|| String::new());

    input {
        oninput: move |e: InputEvent| query.set(e.value()),
        onkeydown: move |e: KeyboardEvent| {
            if e.key == Key::Enter {
                on_search(SearchEvent {
                    query: query(),
                    filters: vec![],
                });
            }
        }
    }
}

// Parent listens
#[component]
fn App() {
    SearchBar {
        on_search: move |e: SearchEvent| {
            log::info!("Searching: {}", e.query);
            perform_search(e.query);
        }
    }
}
```

---

## Cross-Platform Mapping

| rye Event | Web (DOM) | Native (winit) | Mobile |
|---|---|---|---|
| `MouseEvent::Click` | `click` | `WindowEvent::MouseInput` | `TouchEvent::Tap` |
| `MouseEvent::MouseMove` | `mousemove` | `WindowEvent::CursorMoved` | `TouchEvent::TouchMove` |
| `KeyboardEvent::KeyDown` | `keydown` | `WindowEvent::KeyboardInput` | (varies) |
| `InputEvent::Input` | `input` | (text input event) | (text input event) |
| `FocusEvent::Focus` | `focus` | (window focus) | (view focus) |
| `TouchEvent::TouchStart` | `touchstart` | (mapped from mouse) | `touchesBegan` |
| `ScrollEvent::Scroll` | `scroll` | `WindowEvent::MouseWheel` | (scroll gesture) |
| `WindowEvent::Resize` | `resize` | `WindowEvent::Resized` | `viewDidLayoutSubviews` |

Each renderer translates platform events into rye's typed events.

---

*This document defines the event system. **Implemented** in `rye-core/src/event_delegation.rs` (root delegation, handler registry), `rye-html/src/events.rs` (DOM event mapping), and `rye-core/src/component.rs` (typed event types — `MouseEvent`, `KeyboardEvent`, `InputEvent`, `FocusEvent`, `TouchEvent`, `ScrollEvent`, `WindowEvent`).*
