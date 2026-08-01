# rye — Positioning & Strategy

> Where the library fits, what problems it solves, and how to navigate Wasm's trade-offs.

---

## Value Proposition

> **The Rust UI framework for teams that already use Rust — full-stack type safety, cross-platform rendering, and fine-grained reactivity without the virtual DOM tax.**

---

## Where rye Is Genuinely Useful

### 1. Full-stack Rust apps — The killer use case
One language, one type system, one codebase for server + client. `rye-ssr` + `rye-html` + `rye-core` already supports this. The #1 reason teams pick Rust+Wasm UI frameworks over React — they already have a Rust backend and don't want to maintain a separate JS/TS frontend.

**Target**: Fintech, embedded dashboards, IoT control panels, internal tools for Rust-heavy companies.

### 2. Cross-platform from one codebase
`rye-desktop` (wgpu), `rye-html` (web), `rye-mobile` targets mean one component tree renders to native GPU, browser DOM, or mobile. No React Native + Electron + React web split.

**Target**: Apps that need web + desktop + mobile from one team — logistics, field service tools, industrial monitoring.

### 3. Computation-heavy UIs
Apps where the UI is tightly coupled to heavy computation — data visualization, audio/video editors, CAD tools, game editors, scientific dashboards. Wasm's compute speed actually matters here.

**Target**: Trading terminals, audio waveform editors, 3D model viewers, real-time data dashboards.

### 4. Security-critical / sandboxed apps
Rust's memory safety + Wasm's sandbox = double safety layer. No prototype pollution, no supply chain JS vulnerabilities, no `eval` injection surface.

**Target**: Government, healthcare, banking interfaces where audit and safety matter.

---

## Strategic Decisions — What Was Done

### ✅ Lean into SSR + hydration
SSR sidesteps Wasm's startup problem — the user sees HTML immediately, Wasm hydrates in the background. `rye-ssr` with streaming SSR, progressive hydration, edge rendering, and SSG is fully implemented.

### ✅ Minimize DOM bridge calls
`BatchRenderer` groups operations — a single JS call applies all pending mutations. `rye-html/src/batch.rs` implements the DOM batch protocol. The fewer Wasm→JS crossings, the faster the app.

### ✅ Ship a tiny runtime
Arena allocator, code splitting, `wasm-opt -Oz`, minimal `web-sys` features, and tree-shaking all implemented. Target **<80 KB gzipped** for a hello world app is achievable.

### ✅ Provide great DX
Hot reload is implemented via `rye-cli` dev server with WebSocket HMR. Also includes:
- Fast incremental Wasm rebuilds (watch mode)
- Source maps for Wasm (debug builds with `DWARF` info)
- Clear error messages from `template!` and `#[component]` macros
- A dev server that auto-rebuilds on save

### ✅ Make interop seamless
JS interop via `rye-html/src/js_interop.rs` — `use_js_library()` escape hatch, Web Components interop (`rye-html/src/web_components.rs`), and ergonomic `wasm-bindgen` imports.

### ✅ Own the "Rust full-stack" story
Routing (`rye-router`), data fetching (`Resource`), SSR (`rye-ssr`), server actions (`rye-core/src/server_action.rs`), and deployment tooling (`rye-core/src/tooling/deploy.rs`) provide a cohesive full-stack DX.

---

## What Was Avoided (Correctly)

### Avoided: Re-implementing the DOM in Wasm
Some frameworks try to build a virtual DOM in Wasm and diff there. This is the worst of both worlds — you pay Wasm overhead AND DOM bridge overhead. Instead, do **fine-grained reactivity** (the signals approach) — update only the exact DOM node that changed, skip diffing entirely.

### Avoided: Competing with React on ecosystem
React has 100k+ npm packages. You will never match that. Don't try. Instead, provide excellent JS interop so people can use existing libraries when needed, and build a focused set of high-quality Rust-native components.

### Avoided: Requiring Wasm for everything
Consider a **native SSR-only mode** for content sites (blogs, docs, marketing). Not every page needs client-side interactivity. `SsrRenderer` can render full pages on the server with zero Wasm shipped to the client.

### Avoided: Over-abstracting the renderer
The `Renderer` trait is good, but don't make it so abstract that every backend needs 500 lines of boilerplate. Provide default implementations and macros that generate most of it.

### Avoided: Ignoring the JS build tooling
Even though the runtime is Rust, users will interact with npm, CSS tooling, asset pipelines, and deployment platforms. Integrate with existing tooling rather than building everything from scratch.

---

## Wasm Trade-offs — Where There's No Benefit Or It's Worse

### DOM manipulation — Worse
Every DOM operation from Wasm is a bridge call through JS. Setting `element.class_name`, reading `get_bounding_client_rect`, calling `addEventListener` — all cross the Wasm→JS boundary. Pure JS does this directly. For UI-heavy apps, this is most of what you do.

### Startup time — Worse
- JS: parse text → execute. Fast for small files.
- Wasm: download `.wasm` binary → validate → compile → instantiate → load JS glue code. Can add hundreds of milliseconds before the app is interactive.

### Bundle size — Worse
A minimal Rust+Wasm app is typically 50-150 KB of `.wasm` + JS glue, even for "hello world". A minimal React app with tree-shaking can be smaller. Rust's standard library and `web-sys` bindings add weight that's hard to strip.

### Simple CRUD apps — No benefit
If the app is forms, buttons, API calls, and list rendering — JS is already fast enough. The bottleneck is the network and DOM, not computation. Wasm adds complexity with zero perceptible speedup.

### Debugging — Worse
- JS: full source maps, browser DevTools, breakpoints, stack traces — all native.
- Wasm: raw binary or wasm-decoded text. Stack traces show Wasm function indices, not Rust function names (unless built with debug info, which bloats the binary). DevTools support is improving but still far behind JS.

### Text processing — No benefit or worse
JavaScript's V8 engine is heavily optimized for string operations. Wasm doesn't have a built-in string type — strings must be encoded/decoded when crossing the JS bridge. For string-heavy work, JS can actually be faster.

### Async / event-driven code — No benefit
Promises, `setTimeout`, event listeners, `fetch` — these are browser APIs called through JS anyway. Wasm doesn't make them faster. The async runtime (Rust futures) adds overhead compared to JS's native event loop.

### Hot reload / dev iteration — Worse
- JS: save file → instant hot reload.
- Wasm: save file → recompile Rust → recompile to Wasm → re-instantiate. Even with `trunk` or `wasm-pack` watch mode, it's seconds vs milliseconds.

### SEO / server rendering — No benefit
SSR happens on the server (Node.js or Rust native). Wasm is irrelevant there. On the client, Wasm content is rendered to DOM after hydration — search engines may not index it well without proper SSR.

### Memory usage — Can be worse
Wasm linear memory is pre-allocated and grows in large pages (64KB blocks). JS engines manage memory more granularly. A Rust+Wasm app can use more memory than equivalent JS for the same UI.

---

## Fixes For The "No Benefit Or Worse" Points — All Implemented

### DOM manipulation → Batch + fine-grained
- ✅ **Batch**: `BatchRenderer` groups operations — a single JS call applies all pending mutations via `rye-html/src/batch.rs`.
- ✅ **Fine-grained**: Signal system updates only the node bound to a signal — no virtual DOM diffing. `rye-signals` crate.
- ✅ **Direct DOM calls**: JS shim generation at compile time for hot paths reduces per-call overhead.

### Startup time → SSR + streaming hydration
- ✅ Server renders HTML, user sees content immediately (`rye-ssr`).
- ✅ Wasm loads and hydrates in the background (`rye-core/src/hydration.rs`, `rye-ssr/src/server/progressive_hydration.rs`).
- ✅ **Streaming compilation**: `WebAssembly.instantiateStreaming()` supported.
- ✅ **Code splitting**: `rye-core/src/code_split.rs` splits the `.wasm` into a small initial module + lazy-loaded chunks for routes.

### Bundle size → Aggressive optimization
- ✅ `wasm-opt -Oz` in release builds (shrinks 20-40%).
- ✅ Arena allocator in `rye-core/src/alloc.rs` (saves ~10KB).
- ✅ Only enabled `web-sys` features are used.
- ✅ `panic = "abort"` removes unwind tables in release.
- ✅ Tree-shake unused `rye-*` crates at the workspace level.

### Debugging → Source maps + dev tooling
- ✅ Debug builds with `DWARF` debug info → browser DevTools show Rust function names.
- ✅ `rye-devtools` crate — signal graph, component tree, re-render hotspots.
- ✅ `rye inspect` CLI command traces signal dependencies (`rye-core/src/tooling/inspect.rs`).

### Dev iteration / hot reload → Incremental compilation
- ✅ `cargo` incremental compilation + `wasm-pack build --dev` (fast, no optimization).
- ✅ Custom hot-reload protocol: watch `.rs` files → recompile changed crate → send `.wasm` patch to browser → re-instantiate only changed module (`rye-core/src/tooling/hot_reload.rs`, `rye-cli`).
- ✅ Template-only hot swap for `template!` macro changes without full recompile.

### String processing → Minimize bridge crossings
- ✅ Text kept in Wasm memory, only written to DOM on signal change.
- ✅ SIMD text processing in `rye-core/src/perf/simd.rs`.
- ✅ String interning to avoid repeated allocations.
- ✅ Form input batch read operations.

### Memory usage → Right-size allocations
- ✅ `bumpalo` for short-lived allocations (render passes) in `rye-core/src/alloc.rs`.
- ✅ Arena allocation for component tree (reuse memory across renders).
- ✅ Memory profiler in `rye-core/src/perf/memory_profiler.rs`.

---

## Summary — Where rye Wins

| Area | Verdict |
|------|---------|
| Full-stack Rust (server + client) | ✅ Strong win — no type drift |
| Cross-platform (web + desktop + mobile) | ✅ Strong win — one codebase |
| Computation-heavy UIs | ✅ Strong win — Wasm compute speed |
| Security-critical apps | ✅ Strong win — memory safety + sandbox |
| SSR-first content sites | ✅ Win — Rust server, zero Wasm shipped |
| Fine-grained reactivity | ✅ Win — no virtual DOM tax |
| Ecosystem / third-party libs | ✅ Addressed — JS interop via `rye-html/src/js_interop.rs`, Web Components interop |
| Bundle size | ✅ Addressed — arena allocator, code splitting, wasm-opt, tree-shaking |
| Dev tooling / hot reload | ✅ Addressed — `rye-cli` dev server, HMR, `rye inspect` |
| Simple CRUD apps | Neutral — no compelling advantage (by design) |

---

## Market Positioning

Don't market rye as "faster React." Market it as:

1. **No TypeScript↔Backend type drift** — one Rust schema, end to end.
2. **One codebase, three platforms** — web, desktop, mobile.
3. **Fine-grained reactivity** — no virtual DOM, no diffing overhead.
4. **Memory safety + small attack surface** — no JS supply chain vulnerabilities.
5. **SSR-first** — fast first paint, progressive hydration.

This is a real, defensible niche — the same one Leptos and Dioxus are successfully carving out.
