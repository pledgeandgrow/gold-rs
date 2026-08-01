# Routing System Design

> Goal 21 — File-based AND code-based routing: nested routes with typed params, route guards, lazy-loaded routes, SSR-aware navigation, type-safe links.

---

## Design Goals

- **Dual mode** — File-based (convention) or code-based (explicit)
- **Type-safe** — Route params are typed, links are validated at compile time
- **Nested** — Routes can nest with shared layouts
- **Guards** — Auth/data-loading guards before route activation
- **Lazy loading** — Code splitting per route
- **SSR-aware** — Works on server (initial route) and client (navigation)

---

## Code-Based Routing

### Define routes

```rust
use rye::prelude::*;
use rye::router::{Router, Route, Link};

#[component]
fn App() {
    Router {
        routes: routes![
            Route { path: "/", Home {} },
            Route { path: "/about", About {} },
            Route {
                path: "/users/:id",
                guard: require_auth,
                User { id: param!("id") }
            },
            Route {
                path: "/users/:id/posts/:post_id",
                UserPost {}
            },
            Route {
                path: "/dashboard",
                guard: require_auth,
                layout: DashboardLayout,
                children: routes![
                    Route { path: "", DashboardHome {} },
                    Route { path: "settings", Settings {} },
                    Route { path: "billing", Billing {} },
                ],
            },
            Route { path: "/*", NotFound {} },
        ],
    }
}
```

### Typed route params

```rust
// Route: /users/:id
#[component]
fn User() {
    let params = use_params::<UserParams>();
    // params.id is String (from :id)
    let user = use_resource(move || {
        let id = params.id.clone();
        async move { fetch_user(id).await }
    });

    div {
        h1 { "User: " {user().map(|u| u.name).unwrap_or_default()} }
    }
}

#[derive(Params)]
struct UserParams {
    id: String,
}

// Route: /users/:id/posts/:post_id
#[derive(Params)]
struct UserPostParams {
    id: String,
    post_id: String,
}
```

### Typed params with parsing

```rust
#[derive(Params)]
struct UserParams {
    // Auto-parsed from String to i32
    #[param(parse)]
    id: i32,
}
```

### Query params

```rust
#[component]
fn Search() {
    let query = use_query::<SearchQuery>();
    // query.q is Option<String>
    // query.page is Option<u32> (auto-parsed)

    div {
        input {
            value: {query.q.clone().unwrap_or_default()},
            oninput: move |e| navigate(&format!("/search?q={}&page=1", e.value())),
        }
    }
}

#[derive(QueryParams)]
struct SearchQuery {
    q: String,
    #[query(default = 1)]
    page: u32,
}
```

---

## File-Based Routing

### Convention

```
src/pages/
├── index.rs           →  /
├── about.rs           →  /about
├── users/
│   ├── index.rs       →  /users
│   ├── [id].rs        →  /users/:id
│   └── posts/
│       └── [post_id].rs →  /users/:id/posts/:post_id  (wait, this needs both)
├── dashboard/
│   ├── index.rs       →  /dashboard
│   ├── settings.rs    →  /dashboard/settings
│   └── billing.rs     →  /dashboard/billing
└── [...404].rs        →  /* (catch-all)
```

### How it works

- The CLI scans `src/pages/` during build
- Generates a route table from file names
- `[param]` → `:param` route parameter
- `[...name]` → `*name` catch-all
- `index.rs` → the index route for that directory
- Nested directories → nested routes with shared layout (if `_layout.rs` exists)

### Layout files

```
src/pages/
├── dashboard/
│   ├── _layout.rs     →  shared layout for dashboard/* routes
│   ├── index.rs       →  /dashboard
│   ├── settings.rs    →  /dashboard/settings
│   └── billing.rs     →  /dashboard/billing
```

```rust
// _layout.rs
#[component]
fn DashboardLayout(props: LayoutProps) {
    div {
        class: "dashboard",
        Sidebar {}
        main {
            {props.children}  // route content rendered here
        }
    }
}
```

---

## Route Guards

```rust
/// A route guard — runs before route activation.
/// Returns Ok(()) to proceed, Err(redirect) to redirect.
pub trait RouteGuard: 'static {
    fn check(&self) -> Result<(), Redirect>;
}

// Example: auth guard
fn require_auth() -> Result<(), Redirect> {
    let auth = use_context_signal::<AuthState>();
    if auth().user.is_some() {
        Ok(())
    } else {
        Err(Redirect::to("/login"))
    }
}

// In route definition:
Route {
    path: "/dashboard",
    guard: require_auth,
    Dashboard {}
}

// Multiple guards:
Route {
    path: "/admin",
    guards: [require_auth, require_admin],
    Admin {}
}
```

### Data-loading guards

```rust
// Load data before route renders
fn load_user_data() -> Result<(), Redirect> {
    let params = use_params::<UserParams>();
    let user = use_resource(move || {
        let id = params.id.clone();
        async move { fetch_user(id).await }
    });

    // Wait for data to load (Suspense handles the loading state)
    Ok(())
}

Route {
    path: "/users/:id",
    guards: [require_auth, load_user_data],
    User {}
}
```

---

## Lazy-Loaded Routes

```rust
// Code-based lazy loading
Route {
    path: "/dashboard",
    lazy: || async {
        // This code is code-split into a separate WASM module
        let module = import!("dashboard.wasm").await;
        module::Dashboard
    },
}

// File-based: add .lazy.rs extension
// src/pages/dashboard.lazy.rs → automatically code-split
```

---

## Type-Safe Links

```rust
// Type-safe link — validated at compile time
Link {
    to: Route::User { id: 42 },  // Type-checked against route definition
    "View User"
}

// With query params
Link {
    to: Route::Search { q: "rust".to_string(), page: 1 },
    "Search for Rust"
}

// Active state
Link {
    to: Route::About,
    active_class: "nav-link-active",
    "About"
}
```

### Route enum (auto-generated)

```rust
// Auto-generated from route definitions
pub enum Route {
    Home,
    About,
    User { id: String },
    UserPost { id: String, post_id: String },
    Search { q: String, page: u32 },
    NotFound,
}

impl Route {
    fn to_path(&self) -> String {
        match self {
            Route::Home => "/".to_string(),
            Route::User { id } => format!("/users/{}", id),
            // ...
        }
    }
}
```

---

## Navigation API

```rust
/// Programmatic navigation.
pub fn navigate(to: &str);
pub fn navigate_to<T: RoutePath>(route: T);
pub fn back();
pub fn forward();
pub fn redirect(to: &str);

// Usage:
button {
    onclick: move |_| navigate("/dashboard"),
    "Go to Dashboard"
}

// Type-safe:
button {
    onclick: move |_| navigate_to(Route::User { id: 42 }),
    "View User 42"
}
```

---

## SSR-Aware Routing

```rust
// On the server:
// 1. Parse the URL from the request
// 2. Match the route
// 3. Run guards (async)
// 4. Render the matched component to HTML
// 5. Send HTML with hydration data

// On the client:
// 1. Read the initial URL
// 2. Match the route (same route table)
// 3. Hydrate the server-rendered HTML
// 4. Subsequent navigations are client-side (no page reload)
```

---

## Comparison with Competitors

| Feature | React Router | Vue Router | Dioxus | Leptos | rye |
|---|---|---|---|---|---|
| File-based routing | No (needs Next) | No (needs Nuxt) | No | No | Yes |
| Code-based routing | Yes | Yes | Yes | Yes | Yes |
| Typed params | No (TS only) | No (TS only) | No | Yes | Yes |
| Route guards | Yes | Yes | No | No | Yes |
| Lazy loading | Yes (React.lazy) | Yes | No | No | Yes |
| Type-safe links | No | No | No | No | Yes |
| SSR-aware | Yes (with SSR) | Yes (with SSR) | No | Yes | Yes |
| Nested routes | Yes | Yes | No | Yes | Yes |

---

*This document defines the routing system. **Implemented** in `rye-router` crate — code-based and file-based routing, typed params (`#[derive(Params)]`), query params (`#[derive(QueryParams)]`), route guards, lazy-loaded routes, type-safe links, SSR-aware navigation, nested routes with shared layouts.*
