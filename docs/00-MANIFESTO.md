# The Manifesto — A Rust UI Framework for Everyone

> **Working name:** `rye` — *Rust, Yield, Everywhere.*
> A framework that yields to no one in performance, yields control to the developer, and runs everywhere.

---

## Core Philosophy

We believe the web and native development world deserves a UI framework that doesn't force you to choose between **safety**, **performance**, **ergonomics**, and **reach**. You can have all four. `rye` is our proof.

### The Five Pillars

#### 1. Safety Without Compromise
Rust's type system and ownership model are not obstacles to work around — they are features that eliminate entire classes of bugs at compile time. No null reference exceptions, no undefined behavior, no data races. The framework leverages this to provide **compile-time guarantees** about your UI that JavaScript frameworks simply cannot offer.

- Template syntax errors caught at compile time, not runtime
- Props type-checked by the compiler — no PropTypes, no runtime validation
- State transitions verified by the type system
- No "cannot read property of undefined" — ever

#### 2. Performance by Default
Developers should not need to be performance experts to build fast apps. The framework's default behavior should be the optimal behavior.

- Fine-grained reactivity — only the DOM nodes that depend on changed state update, never whole components
- Compile-time template analysis — static parts of the UI are generated once, only dynamic bindings are tracked
- Zero-cost abstractions — what you write is what runs, no hidden overhead
- WASM target — near-native execution speed in the browser
- Native GPU rendering — no WebView tax on desktop

#### 3. Ergonomics That Feel Like Home
A framework is only as good as its developer experience. We study what developers love about React, Vue, Svelte, and Solid — and we bring those ergonomics to Rust without the pain points.

- No "rules of hooks" — create signals anywhere, anytime
- No manual memoization — dependency tracking is automatic
- Familiar template syntax — HTML-like, readable, no deep Rust knowledge required to start
- World-class error messages — no opaque macro panics, no guessing what went wrong
- Hot reloading — see changes instantly, state preserved where possible
- Zero-config CLI — `rye new`, `rye dev`, `rye build`, done

#### 4. True Cross-Platform, One Codebase
Write your UI once. Run it on web (WASM), desktop (native GPU), mobile (native GPU), and server (SSR). Not "similar code" — the **same code**. The framework abstracts platform differences without hiding them when you need control.

- Web: WASM + DOM, full browser API access when needed
- Desktop: Native GPU rendering via `wgpu`, no WebView dependency
- Mobile: Same GPU renderer, adapted for touch and mobile lifecycle
- Server: SSR with streaming, hydration on client
- Conditional platform code: `#[cfg(target = "web")]` or runtime `use_platform()`

#### 5. Batteries Included, Not Imposed
The framework ships with everything you need for a production app: routing, state management, forms, i18n, animations, testing, devtools. But none of it is mandatory. Each feature is a separate crate that can be opted out of.

- Routing, state, forms, i18n, animations — all official, all optional
- Headless UI primitives — accessible, unstyled, composable
- Devtools — component inspector, signal viewer, profiler
- Testing framework — unit, integration, snapshot, E2E hooks
- CLI — scaffold, develop, build, test, deploy, add packages

---

## Design Principles

### Principle 1: The Compiler is Your Friend
We push as much work as possible to compile time. Template parsing, style validation, route type-checking, i18n message extraction — all happen at build time. This means:

- Faster runtime (less work to do)
- Smaller bundles (less code to ship)
- Better errors (caught before deployment)
- More confidence (the compiler verified it)

### Principle 2: Progressive Complexity
Simple things should be simple. A counter component should be 5 lines. A todo app should be 30 lines. Complex things should be possible — but they shouldn't be the default experience.

```
Level 1: template! macro + signals (beginner)
Level 2: components + props + context (intermediate)
Level 3: custom renderers + plugins + SSR (advanced)
Level 4: framework internals contribution (expert)
```

### Principle 3: No Magic, Only Power
Every abstraction should be understandable. When something goes wrong, the developer should be able to trace through the code and understand what happened. We avoid:

- Implicit global state that can't be traced
- Code generation that hides control flow
- "Smart" re-rendering that's impossible to debug
- Macros that expand to thousands of lines

Instead, we provide:
- Devtools that show exactly what happened and why
- Source maps for all generated code
- Documentation for every public API
- Examples for every pattern

### Principle 4: Interop First
The framework must play well with the existing ecosystem. No walled garden.

- Web: Full interop with JavaScript libraries via `wasm-bindgen`
- Desktop: System APIs via platform crates
- CSS: Works with existing CSS, Tailwind, CSS-in-JS approaches
- Build: Integrates with `cargo`, `wasm-pack`, existing Rust tooling

### Principle 5: Accessibility is Not Optional
Every built-in component, every pattern, every example must be accessible by default. The framework generates the correct ARIA attributes, semantic HTML, and platform accessibility tree nodes automatically.

- Screen reader support on all platforms
- Keyboard navigation built into all interactive components
- Focus management for route changes, modals, and dynamic content
- Color contrast validation in dev mode

---

## What We Are Not

- **Not a JavaScript framework ported to Rust** — We leverage Rust's strengths, not mimic JS's weaknesses
- **Not a WebView wrapper** — Native rendering is a first-class target, not an afterthought
- **Not a compiler-only solution** — We use runtime reactivity where it provides the best DX
- **Not an opinion monopoly** — You can swap routing, state, styling for alternatives
- **Not a one-person project** — Community-driven, RFC-based, transparent governance

---

## The Promise

> **If you know React or Vue, you can be productive in `rye` in one afternoon.**
> **If you know Rust, you have superpowers.**
> **If you know both, you are unstoppable.**

---

## Target Audience

1. **Rust developers** who want to build UIs without leaving Rust
2. **Web developers** (React/Vue/Svelte) looking for better performance and safety
3. **Cross-platform teams** tired of maintaining separate web/desktop/mobile codebases
4. **Performance-critical applications** where every millisecond matters
5. **Security-critical applications** where memory safety is non-negotiable

---

## Success Criteria — Status

All 150 goals across 14 phases are now implemented. The success criteria have been addressed as follows:

- ✅ **Ergonomics** — `#[component]` macro, implicit `template!`, prelude, event shorthand all implemented (`rye-macros`, `rye-core`)
- ✅ **WASM bundle size** — Arena allocator, code splitting, `wasm-opt` pipeline, minimal `web-sys` bindings, string interning all implemented (`rye-core/src/alloc.rs`, `rye-core/src/code_split.rs`, `rye-core/src/perf/`)
- ✅ **Performance** — Fine-grained signals, batched DOM protocol, bridge call counter, SIMD text processing, WASM threading all implemented (`rye-signals`, `rye-html/src/batch.rs`, `rye-core/src/perf/`)
- ✅ **Cross-platform** — Web (`rye-html`), desktop (`rye-desktop`), mobile (`rye-mobile`) all render from one component tree
- ✅ **Error messages** — Custom diagnostic layer with Levenshtein suggestions, error codes R001–R799 (`rye-macros/src/template.rs`)
- ✅ **Testing** — `TestRenderer`, event simulation, snapshot testing, property-based testing, a11y testing, mutation testing, contract testing, security auditing all implemented (`rye-testing`, `rye-core/src/testing/`)
- ✅ **SSR + hydration** — `rye-ssr` with streaming SSR, progressive hydration, edge rendering, SSG, server caching, prefetching all implemented

---

*This manifesto is a living document. The five pillars have been fully realized across 14 phases of implementation (Goals 1–150).*
