# Async Model Design

> Goal 19 — `Resource` and `Suspense` primitives: `use_resource` for async data fetching with automatic cancellation, `<Suspense>` for loading states, `<ErrorBoundary>` for error handling, streaming SSR support.

---

## Design Goals

- **Automatic cancellation** — Old fetches are cancelled when dependencies change
- **Suspense boundaries** — Declarative loading states
- **Error boundaries** — Declarative error handling
- **Streaming SSR** — Send HTML chunks as async data resolves
- **No manual cleanup** — Resources auto-cancel on component unmount

---

## Primitives

### Resource<T> — Async Data

```rust
/// Async data that re-fetches when signal dependencies change.
/// Automatically cancels previous fetch on re-fetch.
pub struct Resource<T: Clone + 'static> { /* ... */ }

pub enum ResourceState<T: Clone> {
    /// Loading — no data yet, or re-fetching.
    Pending,
    /// Data loaded successfully.
    Ready(T),
    /// Fetch failed.
    Error(String),
}

impl<T: Clone + 'static> Resource<T> {
    /// Create a resource. The closure returns a future that is
    /// executed immediately and re-executed when signal deps change.
    pub fn new<F, Fut>(fetch: F) -> Self
    where
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = Result<T, String>> + 'static;

    /// Get the current state.
    pub fn get(&self) -> ResourceState<T>;

    /// Force a re-fetch.
    pub fn refresh(&self);
}
```

### use_resource hook

```rust
/// Create a resource in the current component scope.
/// Auto-cancels on unmount.
pub fn use_resource<T, F, Fut>(fetch: F) -> Resource<T>
where
    T: Clone + 'static,
    F: Fn() -> Fut + 'static,
    Fut: Future<Output = Result<T, String>> + 'static;
```

---

## Suspense Component

```rust
/// Suspense boundary — shows fallback while resources are pending.
///
/// When any Resource inside the Suspense is Pending,
/// the fallback is rendered instead.
#[component]
fn Suspense(props: SuspenseProps) {
    // props.fallback: Element
    // props.children: Vec<Element>
    // Framework tracks which resources are read inside children.
    // If any are Pending, render fallback instead.
}
```

### Usage

```rust
#[component]
fn UserProfile(props: UserProfileProps) {
    // Fetch user data reactively
    let user = use_resource(move || {
        let id = props.id;
        async move { fetch_user(id).await }
    });

    // Fetch user's posts
    let posts = use_resource(move || {
        let id = props.id;
        async move { fetch_posts(id).await }
    });

    div {
        Suspense {
            fallback: div { "Loading profile..." },
            div {
                h2 { {user().map(|u| u.name).unwrap_or_default()} }
                Suspense {
                    fallback: div { "Loading posts..." },
                    For each(post in posts().unwrap_or_default()) {
                        key: post.id,
                        Post { post: post.clone() }
                    }
                }
            }
        }
    }
}
```

### Nested Suspense

Each Suspense boundary is independent. A nested Suspense can show its own fallback while the outer content is already loaded:

```
┌─ Suspense (outer) ─────────────────────────────────┐
│  "Loading profile..."                              │  ← shown while user is Pending
│                                                     │
│  ┌─ Suspense (inner) ───────────────────────────┐  │
│  │  "Loading posts..."                          │  │  ← shown while posts are Pending
│  │                                               │  │
│  │  Post 1                                       │  │  ← shown when posts are Ready
│  │  Post 2                                       │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

---

## ErrorBoundary Component

```rust
/// Error boundary — catches errors from children.
/// If a child resource fails or a component panics,
/// the error fallback is rendered.
#[component]
fn ErrorBoundary(props: ErrorBoundaryProps) {
    // props.fallback: impl Fn(Error) -> Element
    // props.children: Vec<Element>
    // props.on_error: impl Fn(Error)  (optional, for logging)
}
```

### Usage

```rust
#[component]
fn App() {
    ErrorBoundary {
        fallback: move |err| div {
            class: "error",
            h2 { "Something went wrong" }
            p { {err.to_string()} }
            button {
                onclick: reload_page,
                "Reload"
            }
        },
        on_error: move |err| log::error!("Component error: {}", err),

        Router {
            Route { path: "/", Home {} }
            Route { path: "/profile/:id", UserProfile {} }
        }
    }
}
```

---

## Streaming SSR

With streaming SSR, the server sends HTML as resources resolve:

```rust
// Server-side: streaming response
async fn handler(req: Request) -> Response {
    let stream = render_to_stream(|| {
        template! {
            html {
                head { title { "My App" } }
                body {
                    div {
                        id: "app",
                        Suspense {
                            fallback: div { "Loading..." },
                            UserProfile { id: 1 }
                        }
                    }
                }
            }
        }
    });

    Response::stream(stream)
}
```

### Streaming flow

```
Time 0ms:   Server sends <html><head>...</head><body><div id="app">
Time 0ms:   Server sends <div>Loading...</div> (Suspense fallback)
Time 50ms:  User data resolves
Time 50ms:  Server sends <script>rye_hydrate("user-1", {name: "Jane"})</script>
Time 50ms:  Server sends </div></body></html>
Time 50ms:  Client hydrates, replaces fallback with real content
```

### Hydration markers

The SSR renderer embeds markers in the HTML:

```html
<div id="app">
  <!--rye-suspense-start:1-->
  <div>Loading...</div>
  <!--rye-suspense-end:1-->
</div>
<script>
  // When user data resolves, server sends:
  rye_hydrate_suspense(1, {name: "Jane"});
</script>
```

---

## Resource with caching

```rust
/// Cached resource — results are cached by key.
/// Re-fetching with the same key returns cached result.
pub fn use_cached_resource<K, T, F, Fut>(
    key: impl Fn() -> K,
    fetch: F,
) -> Resource<T>
where
    K: Hash + Eq + Clone + 'static,
    T: Clone + 'static,
    F: Fn(K) -> Fut + 'static,
    Fut: Future<Output = Result<T, String>> + 'static,
```

### Usage

```rust
let user = use_cached_resource(
    || props.user_id,  // cache key
    |id| async move { fetch_user(id).await },
);

// If user_id changes from 1 to 2 and back to 1,
// the second fetch for id=1 returns cached result instantly.
```

---

## Comparison with Competitors

| Feature | React | Vue | SolidJS | Leptos | rye |
|---|---|---|---|---|---|
| Async primitive | (manual) | (manual) | createResource | Resource | Resource |
| Auto-cancellation | No (manual) | No | Yes | Yes | Yes |
| Suspense | Yes | No | No | Suspense | Suspense |
| Error boundary | Yes (class) | No | No | ErrorBoundary | ErrorBoundary |
| Streaming SSR | Yes (React 18) | No | No | Yes | Yes |
| Resource caching | No (manual) | No | No | No | Yes (use_cached_resource) |

---

*This document defines the async model. **Implemented** in `rye-signals/src/resource.rs` (`Resource<T>`, `ResourceState`, `use_resource`, `use_cached_resource`), `rye-core/src/suspense.rs` (`Suspense`, `ErrorBoundary`), and `rye-ssr/src/streaming.rs` (streaming SSR with hydration markers).*
