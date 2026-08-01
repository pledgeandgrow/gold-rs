# Animation & Transition System Design

> Goal 23 — Declarative animations: `<Transition>` and `<TransitionGroup>` components, enter/leave transitions, FLIP animations for lists, CSS animation integration, spring physics.

---

## Design Goals

- **Declarative** — Animate via components, not imperative API
- **CSS-based** — Leverage CSS transitions/animations for web (GPU-accelerated)
- **Spring physics** — Built-in spring animations for natural motion
- **FLIP for lists** — First-Last-Invert-Play for smooth list reordering
- **Cross-platform** — Works on web (CSS) and native (GPU interpolation)
- **Performant** — No layout thrashing, transform/opacity only

---

## Transition Component

### Enter/Leave transitions

```rust
use rye::prelude::*;
use rye::animations::Transition;

#[component]
fn Modal(props: ModalProps) {
    let show = use_signal(|| false);

    Transition {
        show: show(),
        name: "fade",  // CSS class prefix: .fade-enter, .fade-leave, etc.
        mode: TransitionMode::InOut,  // or OutIn, or default (simultaneous)

        div {
            class: "modal",
            "Modal content"
        }
    }
}
```

### CSS classes generated

```css
/* .fade-enter — starting state */
.fade-enter {
    opacity: 0;
    transform: translateY(-20px);
}

/* .fade-enter-active — active state (transition applied) */
.fade-enter-active {
    transition: opacity 0.3s, transform 0.3s;
}

/* .fade-leave — starting state for leave */
.fade-leave {
    opacity: 1;
    transform: translateY(0);
}

/* .fade-leave-active — active state for leave */
.fade-leave-active {
    transition: opacity 0.3s, transform 0.3s;
    opacity: 0;
    transform: translateY(-20px);
}
```

### Transition modes

```rust
pub enum TransitionMode {
    /// New element enters first, then old leaves.
    InOut,
    /// Old element leaves first, then new enters.
    OutIn,
    /// Both happen simultaneously (default).
    Default,
}
```

---

## TransitionGroup Component

### List animations

```rust
use rye::animations::TransitionGroup;

#[component]
fn TodoList() {
    let todos = use_signal(|| vec![
        Todo { id: 1, text: "Learn rye" },
        Todo { id: 2, text: "Build app" },
    ]);

    TransitionGroup {
        name: "list",  // CSS class prefix
        tag: "ul",     // wrapper element

        For each(todo in todos()) {
            key: todo.id,
            li {
                class: "todo-item",
                {todo.text}
                button {
                    onclick: move |_| remove_todo(todo.id),
                    "Delete"
                }
            }
        }
    }
}
```

### FLIP animation for list reordering

When list items change position, the FLIP technique animates them smoothly:

```
F: First — record initial position
L: Last — record final position (after DOM update)
I: Invert — apply transform to make element appear at first position
P: Play — remove transform, element animates to final position
```

```rust
TransitionGroup {
    name: "list",
    flip: true,  // Enable FLIP animations
    flip_duration: 300,  // ms

    For each(item in sorted_items()) {
        key: item.id,
        Item { data: item.clone() }
    }
}
```

---

## Spring Physics

### Spring component

```rust
use rye::animations::{Spring, SpringConfig};

#[component]
fn AnimatedCounter() {
    let target = use_signal(|| 0);
    let spring = Spring::new(0, SpringConfig::default());

    // Spring animates toward target
    spring.animate_to(target());

    div {
        h1 { {spring.value()} }  // Smoothly animates
        button { onclick: target += 1, "+" }
        button { onclick: target -= 1, "-" }
    }
}
```

### Spring config

```rust
// Presets
SpringConfig::default()        // Natural (mass: 1, stiffness: 170, damping: 26)
SpringConfig::gentle()         // Slow, smooth
SpringConfig::wobbly()         // Bouncy
SpringConfig::stiff()          // Quick, minimal bounce
SpringConfig::slow()           // Very slow
SpringConfig::molasses()       // Extremely slow

// Custom
SpringConfig {
    mass: 1.0,
    stiffness: 170.0,
    damping: 26.0,
    velocity: 0.0,
    precision: 0.01,
}
```

### Spring on any value

```rust
// Animate position
let x = Spring::new(0.0, SpringConfig::wobbly());
let y = Spring::new(0.0, SpringConfig::wobbly());

div {
    style: {format!("transform: translate({}px, {}px)", x.value(), y.value())},
    onclick: move |_| {
        x.animate_to(100.0);
        y.animate_to(50.0);
    },
    "Click to animate"
}
```

---

## CSS Animation Integration

### Using CSS keyframes

```rust
#[component]
fn Spinner() {
    div {
        class: "spinner",
        // CSS: .spinner { animation: spin 1s linear infinite; }
        //      @keyframes spin { from { transform: rotate(0) } to { transform: rotate(360deg) } }
    }
}
```

### Dynamic animation duration

```rust
let speed = use_signal(|| 1.0);

div {
    class: "spinner",
    style: {format!("animation-duration: {}s", speed())},
    "Loading"
}
```

---

## Native Animations (wgpu)

On native platforms (desktop/mobile), CSS isn't available. The animation system uses GPU interpolation:

```rust
// On native, Spring and Transition use wgpu to interpolate:
// - Transform (translate, scale, rotate)
// - Opacity
// - Color (for backgrounds)
//
// The animation system hooks into the render loop:
// Each frame, spring physics are updated, and the render tree
// is updated with new transform/opacity values.
```

---

## Comparison with Competitors

| Feature | React | Vue | Svelte | Dioxus | rye |
|---|---|---|---|---|---|
| Transition component | External (react-transition-group) | Built-in | Built-in | No | Yes |
| TransitionGroup | External | Built-in | Built-in | No | Yes |
| FLIP animations | External | No | Built-in (flip) | No | Yes |
| Spring physics | External (react-spring) | External | Built-in (svelte/motion) | No | Yes |
| CSS integration | Manual | Built-in | Built-in | No | Yes |
| Native (non-CSS) | N/A | N/A | N/A | No | Yes (wgpu) |

---

*This document defines the animation/transition system. **Implemented** in `rye-animations` crate — `Transition` component (enter/leave, InOut/OutIn modes), `TransitionGroup` with FLIP animations, `Spring` with physics-based animation and presets, CSS keyframe integration, and native wgpu interpolation for non-web platforms.*
