# WASM Binary Size Optimization Strategy

> Goal 6: Research and document strategies to achieve <50KB gzipped for a hello-world app.

---

## Target

| Metric | Target | Current Dioxus | Current Yew | React+ReactDOM |
|---|---|---|---|---|
| Hello world (gzipped) | <50KB | ~80KB | ~100KB | ~45KB |
| TodoMVC (gzipped) | <80KB | ~120KB | ~150KB | ~150KB |
| Full app with router (gzipped) | <120KB | ~180KB | ~220KB | ~200KB |

React ships at ~45KB gzipped for just React+ReactDOM (no app code). Our target of <50KB includes the framework runtime **plus** a hello-world app.

---

## Strategy 1: Modular Crate Architecture

Every feature is a separate crate. The linker strips unused code.

```
rye-core        → required (always linked)     ~15KB gzipped
rye-signals     → required (always linked)     ~5KB gzipped
rye-macros      → compile-time only (no runtime cost)  0KB
rye-html        → optional (web DOM layer)     ~8KB gzipped
rye-router      → optional                     ~4KB gzipped
rye-forms       → optional                     ~3KB gzipped
rye-i18n        → optional                     ~3KB gzipped
rye-animations  → optional                     ~4KB gzipped
rye-devtools    → dev only (stripped in prod)  0KB in prod
rye-testing     → test only (stripped in prod) 0KB in prod
```

**Hello world** links only `rye-core` + `rye-signals` + `rye-html` = ~28KB gzipped + app code (~2KB) = **~30KB**.

### How

- Each crate is independently compilable
- `rye` meta-crate re-exports all, but `cargo` tree-shakes unused crates
- Feature flags on `rye` crate: `features = ["router", "forms"]` — only pulls what's needed
- `#[cfg(feature = "...")]` gates on all optional code paths

---

## Strategy 2: Aggressive Dead Code Elimination

### LTO (Link-Time Optimization)

```toml
# Cargo.toml
[profile.release]
lto = true              # or "fat" for maximum
codegen-units = 1       # single codegen unit = better optimization
opt-level = "z"         # optimize for size (or "s" for balanced)
panic = "abort"         # no unwinding machinery
strip = true            # strip debug symbols
```

### Expected savings

| Optimization | Size reduction |
|---|---|
| LTO=true | ~15-20% |
| codegen-units=1 | ~5-10% |
| opt-level="z" | ~10-15% |
| panic="abort" | ~10-15% |
| strip=true | ~5-10% |
| **Combined** | **~40-60% reduction** |

---

## Strategy 3: `wasm-opt` Post-Processing

Run `wasm-opt` on the final `.wasm` binary:

```bash
wasm-opt -Oz -o output.wasm input.wasm
```

### Passes

| Pass | What it does | Impact |
|---|---|---|
| `-Oz` | Optimize for size (all size passes) | ~10-20% |
| `--strip-debug` | Remove debug sections | ~5-10% |
| `--strip-producers` | Remove producers section | ~1KB |
| `--dce` | Dead code elimination | ~5% |
| `--inlining-optimizing` | Inline small functions | ~5% |

### Integration

The `rye` CLI runs `wasm-opt` automatically during `rye build --target web`:
```bash
rye build --target web
# Internally: cargo build → wasm-bindgen → wasm-opt -Oz → gzip
```

---

## Strategy 4: Minimal `web-sys` Bindings

`web-sys` generates bindings for the entire Web API. By default, pulling in `web-sys` can add hundreds of KB. We must be surgical.

### Approach

- Enable only specific `web-sys` features:
```toml
[dependencies.web-sys]
version = "0.3"
features = [
    "Document", "Element", "Node", "Text", "Window",
    "HtmlElement", "HtmlInputElement", "HtmlButtonElement",
    "Event", "MouseEvent", "KeyboardEvent", "InputEvent",
    "DocumentFragment", "ShadowRoot",
    # Only what we use — nothing more
]
```

- Never use `web-sys = { features = ["all"] }` or broad feature groups
- Audit `web-sys` feature usage in CI — fail if unused features are enabled
- Custom thin wrappers around the handful of DOM APIs we actually need

### Expected impact

| Approach | `web-sys` size (gzipped) |
|---|---|
| All features | ~300KB+ |
| Default features | ~100KB |
| Our curated set | ~15-20KB |

---

## Strategy 5: Avoid `serde` for Simple Types

`serde` + `serde_json` add ~20-30KB gzipped. For simple state serialization (e.g., SSR state transfer), use a minimal serializer:

### Approach

- `rye-serialize` — minimal serializer for framework state (signals, resources)
- Only ~2KB gzipped
- Supports: `String`, `i32`, `i64`, `f64`, `bool`, `Vec<T>`, `HashMap<K,V>`, `Option<T>`
- Users can opt into full `serde` for their app data if needed
- SSR state transfer uses `rye-serialize` by default

---

## Strategy 6: String Interning

DOM tag names, attribute names, and event names are known at compile time. Instead of storing them as `&str` in the binary (each instance adds to size), intern them:

```rust
// Instead of many "div" string literals scattered in code:
pub const DIV: &str = "div";  // single instance in binary
pub const CLASS: &str = "class";
pub const ONCLICK: &str = "onclick";
// etc.
```

The `template!` macro references these constants instead of generating new string literals.

### Expected impact

- ~2-5KB saved (hundreds of duplicate string literals eliminated)

---

## Strategy 7: Tree-Shaking via `#[cfg]` and Generics

Rust's monomorphization can bloat binaries. Mitigate with:

### Generic deduplication

```rust
// Bad: generates separate code for Signal<i32>, Signal<String>, Signal<f64>...
// Good: use trait objects for storage, generics only for API

// Internal storage uses type-erased values
struct SignalInner {
    value: Box<dyn Any>,
    subscribers: Vec<Subscriber>,
}

// Public API is generic but delegates to type-erased internals
impl<T: 'static> Signal<T> {
    fn get(&self) -> T { ... }  // thin wrapper
}
```

### `#[cfg]` gates for platform-specific code

```rust
#[cfg(target_arch = "wasm32")]
fn dom_create_element(tag: &str) -> Node { /* web-sys */ }

#[cfg(not(target_arch = "wasm32"))]
fn dom_create_element(tag: &str) -> Node { /* native renderer */ }
```

---

## Strategy 8: Code Splitting / Dynamic Imports

For larger apps, split the WASM binary:

### Route-level splitting

```rust
// rye-router automatically code-splits lazy routes
Route::lazy("/dashboard", || async {
    // This module is a separate .wasm file, loaded on demand
    let dashboard = import!("../dashboard/mod.rs").await;
    dashboard::Dashboard
})
```

### How

- `wasm-bindgen` supports dynamic imports via `JsValue::from_str()` + `js_sys::Module`
- Each lazy route compiles to a separate `.wasm` file
- Shared dependencies (framework core) are in a shared chunk
- The `rye` CLI manages chunk splitting during build

### Expected impact

- Initial load: only framework core + current route's WASM
- Subsequent routes: loaded on demand, cached by browser
- Example: 200KB total app → 50KB initial + lazy-loaded chunks

---

## Strategy 9: Compression

Always serve WASM with compression:

| Compression | Ratio | Overhead |
|---|---|---|
| gzip | ~70% reduction | Minimal (universal support) |
| brotli | ~80% reduction | Slight CPU overhead |
| zstd | ~75% reduction | Fast decompression |

### CLI integration

```bash
rye build --target web
# Generates:
# - app.wasm          (raw)
# - app.wasm.gz       (gzip)
# - app.wasm.br       (brotli)
# - app.html          (with correct Content-Encoding headers hint)
```

The CLI outputs a `.br` file alongside `.wasm`. The deployment guide shows how to configure the server to serve pre-compressed assets.

---

## Strategy 10: Benchmarking in CI

### Automated size tracking

```yaml
# .github/workflows/size-check.yml
- name: Build hello world
  run: rye new hello --template web && cd hello && rye build --target web

- name: Check WASM size
  run: |
    SIZE=$(gzip -c dist/app.wasm | wc -c)
    echo "Gzipped size: $SIZE bytes"
    if [ $SIZE -gt 51200 ]; then  # 50KB
      echo "::error::WASM bundle exceeds 50KB target ($SIZE bytes)"
      exit 1
    fi
```

### Size budget per crate

Each crate has a size budget. CI fails if a crate exceeds its budget:

| Crate | Budget (gzipped) |
|---|---|
| rye-core | 15KB |
| rye-signals | 5KB |
| rye-html | 8KB |
| rye-router | 4KB |
| rye-forms | 3KB |
| rye-i18n | 3KB |
| rye-animations | 4KB |
| Total (hello world) | 50KB |

---

## Implementation Priority

| Priority | Strategy | Effort | Impact |
|---|---|---|---|
| P0 | Modular crates | Medium | Critical |
| P0 | LTO + opt-level + panic=abort | Low | Critical |
| P0 | Minimal web-sys features | Medium | Critical |
| P0 | wasm-opt post-processing | Low | High |
| P1 | String interning | Low | Medium |
| P1 | Avoid serde for framework state | Medium | Medium |
| P1 | Generic deduplication | Medium | Medium |
| P2 | Code splitting / dynamic imports | High | High (for large apps) |
| P2 | Brotli compression output | Low | Medium |
| P2 | CI size tracking + budgets | Low | Critical (prevents regression) |

---

*This document defines the WASM optimization strategy. **Implemented** across `rye-core/src/alloc.rs` (arena allocator), `rye-core/src/code_split.rs` (code splitting), `rye-core/src/perf/` (SIMD, threading, bridge counter, memory profiler, streaming), `rye-html/src/batch.rs` (DOM batching), and `rye-serialize` (custom serialization).*
