# Testing Framework Design

> Goal 25 — Built-in testing: component unit testing (virtual renderer), signal testing utilities, event simulation, snapshot testing, SSR integration testing, E2E hooks (Playwright integration).

---

## Design Goals

- **Zero setup** — Testing works out of the box, no extra dependencies
- **Virtual renderer** — Test components without a browser or WASM
- **Signal utilities** — Test reactive logic in isolation
- **Event simulation** — Fire events in tests
- **Snapshot testing** — Capture and compare rendered output
- **SSR testing** — Test server-side rendering
- **E2E integration** — Playwright hooks for browser testing

---

## Component Unit Testing

### Render and query

```rust
use rye::prelude::*;
use rye::testing::{TestRenderer, queries};

#[test]
fn counter_increments() {
    let renderer = TestRenderer::new();
    renderer.mount(Counter {});

    // Query the rendered output
    let h1 = queries::get_by_tag(&renderer.root(), "h1");
    assert_eq!(queries::get_all_text(&h1[0]), "Count: 0");

    // Find button and simulate click
    let buttons = queries::get_by_tag(&renderer.root(), "button");
    events::fire_click(&buttons[0]);

    // Assert updated state
    let h1 = queries::get_by_tag(&renderer.root(), "h1");
    assert_eq!(queries::get_all_text(&h1[0]), "Count: 1");
}
```

### Query helpers

```rust
// By tag name
let divs = queries::get_by_tag(&root, "div");

// By class
let cards = queries::get_by_class(&root, "card");

// By attribute
let inputs = queries::get_by_attribute(&root, "type", "text");

// By text content
let elements = queries::get_by_text(&root, "Hello");

// By test ID
let element = queries::get_by_test_id(&root, "submit-btn");
```

### Test IDs

```rust
// In component:
#[component]
fn LoginForm() {
    form {
        input {
            test_id: "email-input",
            type: "text",
        }
        button {
            test_id: "submit-btn",
            type: "submit",
            "Login"
        }
    }
}

// In test:
let btn = queries::get_by_test_id(&root, "submit-btn");
events::fire_click(&btn);
```

---

## Event Simulation

```rust
use rye::testing::events;

// Click
events::fire_click(&element);

// Input
events::fire_input(&element, "hello@example.com");

// Keydown
events::fire_keydown(&element, Key::Enter);

// Focus/blur
events::fire_focus(&element);
events::fire_blur(&element);

// Submit form
events::fire_submit(&form);

// Custom event
events::fire_event(&element, "custom", &payload);
```

---

## Signal Testing

```rust
use rye::signals::{Signal, Memo, Effect, batch};

#[test]
fn signal_tracks_changes() {
    let count = Signal::new(0);
    assert_eq!(count(), 0);

    count.set(5);
    assert_eq!(count(), 5);

    count.update(|v| *v += 1);
    assert_eq!(count(), 6);
}

#[test]
fn memo_recomputes_on_dependency_change() {
    let a = Signal::new(2);
    let b = Signal::new(3);
    let sum = Memo::new(move || a() + b());

    assert_eq!(sum(), 5);

    a.set(10);
    assert_eq!(sum(), 13);

    b.set(20);
    assert_eq!(sum(), 30);
}

#[test]
fn batched_updates_notify_once() {
    let count = Signal::new(0);
    let notifications = Signal::new(0);

    Effect::new(move || {
        count(); // track count
        notifications.update(|n| *n += 1);
    });

    assert_eq!(notifications(), 1); // initial run

    // Without batch: 2 notifications
    count.set(1);
    count.set(2);
    assert_eq!(notifications(), 3);

    // With batch: 1 notification
    batch(|| {
        count.set(3);
        count.set(4);
    });
    assert_eq!(notifications(), 4); // only +1
}

#[test]
fn untracked_reads_dont_register() {
    let count = Signal::new(0);
    let effect_runs = Signal::new(0);

    Effect::new(move || {
        count.get_untracked(); // no tracking
        effect_runs.update(|n| *n += 1);
    });

    assert_eq!(effect_runs(), 1);

    count.set(10);
    assert_eq!(effect_runs(), 1); // still 1 — no re-run
}
```

---

## Snapshot Testing

```rust
use rye::testing::{TestRenderer, snapshot};

#[test]
fn card_renders_correctly() {
    let renderer = TestRenderer::new();
    renderer.mount(Card {
        title: "Test Title".to_string(),
        body: "Test body".to_string(),
    });

    // First run: saves snapshot
    // Subsequent runs: compares against saved snapshot
    snapshot::assert_matches_snapshot(&renderer.root(), "card_basic");

    // Snapshot file: tests/__snapshots__/card_basic.snap
    // Contains the rendered HTML structure
}
```

### Snapshot file format

```
# tests/__snapshots__/card_basic.snap
<div class="card">
  <h2>Test Title</h2>
  <p>Test body</p>
</div>
```

### Updating snapshots

```bash
rpg test --update-snapshots
```

---

## SSR Integration Testing

```rust
use rye::ssr::render_to_string;

#[test]
fn ssr_renders_html() {
    let html = render_to_string(|| {
        template! {
            div {
                class: "app",
                h1 { "Hello, SSR!" }
            }
        }
    });

    assert!(html.contains("<div class=\"app\">"));
    assert!(html.contains("<h1>Hello, SSR!</h1>"));
    assert!(html.contains("data-rye-id")); // hydration markers
}

#[test]
fn ssr_with_signals() {
    let count = Signal::new(42);
    let html = render_to_string(|| {
        template! {
            div { "Count: " {count} }
        }
    });

    assert!(html.contains("Count: 42"));
}
```

---

## E2E Testing (Playwright)

```rust
// tests/e2e/counter_test.rs
use rye::testing::e2e::{Page, expect};

#[rye_e2e_test]
async fn counter_e2e(page: Page) {
    // Navigate to app
    page.goto("http://localhost:8080").await;

    // Check initial state
    expect(page.locator("h1")).to_have_text("Count: 0").await;

    // Click increment
    page.locator("button:has-text('+')").click().await;

    // Verify update
    expect(page.locator("h1")).to_have_text("Count: 1").await;

    // Click 9 more times
    for _ in 0..9 {
        page.locator("button:has-text('+')").click().await;
    }

    expect(page.locator("h1")).to_have_text("Count: 10").await;
}
```

### Running E2E tests

```bash
# Start dev server and run E2E tests
rpg test --e2e

# Or with explicit server
rpg dev --port 8080 &
rpg test --e2e --url http://localhost:8080
```

---

## Test Helpers Summary

| Helper | Purpose |
|---|---|
| `TestRenderer::new()` | Create in-memory renderer |
| `queries::get_by_tag()` | Find elements by tag name |
| `queries::get_by_class()` | Find elements by class |
| `queries::get_by_test_id()` | Find elements by test ID |
| `queries::get_by_text()` | Find elements by text content |
| `events::fire_click()` | Simulate click |
| `events::fire_input()` | Simulate text input |
| `events::fire_keydown()` | Simulate key press |
| `events::fire_submit()` | Simulate form submit |
| `snapshot::assert_matches_snapshot()` | Snapshot comparison |
| `render_to_string()` | SSR testing |

---

## Comparison with Competitors

| Feature | React | Vue | Dioxus | Leptos | rye |
|---|---|---|---|---|---|
| Test renderer | Yes (testing-library) | Yes (@vue/test-utils) | No | No | Yes (built-in) |
| Event simulation | Yes (testing-library) | Yes | No | No | Yes (built-in) |
| Query helpers | Yes (testing-library) | Yes | No | No | Yes (built-in) |
| Snapshot testing | Yes (jest) | Yes (vitest) | No | No | Yes (built-in) |
| Signal testing | N/A | No | No | No | Yes (built-in) |
| SSR testing | Manual | Manual | No | No | Yes (built-in) |
| E2E integration | Playwright | Playwright | No | No | Yes (Playwright hooks) |
| Zero setup | No (needs jest/vitest) | No | No | No | Yes (cargo test) |

---

*This document defines the testing framework. **Implemented** in `rye-testing` crate (`TestRenderer`, query helpers, event simulation, snapshot testing, SSR testing, Playwright E2E hooks) and `rye-core/src/testing/` (`property_testing.rs`, `a11y_testing.rs`, `mutation_testing.rs`, `contract_testing.rs`, `security.rs`).*
