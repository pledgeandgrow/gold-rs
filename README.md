# rye — A Cross-Platform UI Framework for Rust

> **R**ust, **Y**ield, **E**verywhere.

A UI framework that combines Rust's safety, React's ergonomics, Vue's simplicity, SolidJS's performance, and Svelte's developer experience — for web, desktop, mobile, and server from a single codebase.

## Status

**Early-stage / experimental.** This is not a production-ready framework. The
codebase compiles and has a test suite (~1,970 tests passing), but many
features are scaffolds or stubs rather than working implementations. Treat the
feature list below as a **design roadmap**, not a list of proven capabilities.

### What actually works (verified by build + tests)

- **`rye-signals`** — Signal, Memo, Effect, Resource, GlobalSignal with
  automatic dependency tracking. This is the most complete crate.
- **`rye-core`** — Component trait, Renderer trait, Template type, context/DI,
  suspense/error boundaries, hydration markers, server actions, islands.
  Compiles and is unit-tested, but not yet proven in a real app.
- **`rye-macros`** — `template!` macro: proven with 16 tests covering nested
  elements, static/dynamic attributes, event handlers, reactive text, void
  elements, and full SSR pipeline. See known limitations below.
- **`rye-html`** — WASM/DOM renderer via `web-sys`. The demo app targets this.
- **`rye-ssr`** — String-based SSR renderer with hydration markers. Proven
  with 209 tests including void elements, attributes, fragments, and the full
  `template!` → SSR pipeline.
- **`rye-cli` (`rpg`)** — CLI binary with 21 subcommands. Builds and runs.
- **`rye-desktop`** — Native GPU renderer (wgpu + taffy + cosmic-text) with
  real WGSL shaders. Proven with 11 tests: render tree building, taffy
  flexbox layout, style/color parsing, headless GPU init (device + pipeline +
  glyph atlas without a display), and full render-tree-to-layout pipeline.
  Fixed a bug where text nodes weren't getting layout computed.
- **`rye-testing`** — In-memory TestRenderer for unit tests.

### What's proven by dogfooding

- **`template!` macro → SSR pipeline**: The Counter page in `rye-demo` was
  rewritten to use `template!` (49 lines vs 105 lines of hand-built trees).
  It renders correctly through SSR with all classes, headings, and buttons.
- **Desktop GPU renderer core**: Render tree building, taffy layout
  computation, and wgpu device/pipeline initialization all work headless.
  The text layout bug (text nodes skipped by `collect_layouts`) was found and
  fixed through testing.

### Known limitations (being fixed)

- **`template!` macro limitations**: No component invocation, no reactive list
  syntax (`For`/keyed reconciliation), no `if`/`for` in template body, single
  root node only, returns `Element` not `Template`. Signals must be manually
  cloned before each use in dynamic positions (move closures capture by value).
  These gaps prevent converting Dashboard and Todo pages (which use component
  invocations and reactive lists).
- **`rye-mobile` has FFI bindings but is untested on devices.** The crate
  includes ~2,000 LOC of JNI (Android) and ObjC (iOS) bindings in `src/ffi/`,
  plus ~7,000 LOC of config types and manager APIs. It compiles on desktop
  (FFI is cfg-gated), but has never been run on an actual Android or iOS
  device. Treat mobile as **unproven**, not non-existent.
- **No real-world app has been shipped.** The demo renders in a browser via
  WASM, but no app has been deployed or used in production.
- **Clippy has ~240 warnings.** The CI clippy job is advisory (non-blocking)
  until these are cleaned up.
- **`goal.md` marks 250/250 goals as complete.** This is aspirational, not
  verified. The goals are being re-audited against actual implementations.
- **Five crates are minimal stubs** (55–87 LOC each, 0 tests):
  `rye-router`, `rye-forms`, `rye-animations`, `rye-i18n`, `rye-devtools`.
  They define types and module structure but have no real implementation.

## What rye Offers

### Core Framework

- **Fine-grained reactivity** — Signal-based with automatic dependency tracking, no "rules of hooks". `Signal`, `Memo`, `Effect`, `Resource`, `GlobalSignal`, derived selectors, state machines, optimistic updates, cross-tab sync, reactive URL state
- **Compile-time templates** — `template!` macro with HTML-like syntax, optimized at build time. Component-level code generation eliminates dynamic dispatch
- **Cross-platform rendering** — Web (WASM/DOM), Desktop (native GPU via wgpu + taffy + cosmic-text), Mobile (iOS/Android), SSR with streaming
- **Hydration** — Progressive, incremental, island-based, and dual-pass hydration strategies with viewport-priority scheduling
- **Suspense & error boundaries** — Async component support with streaming SSR, retry boundaries, suspense state management
- **Context & DI** — `provide_context`/`use_context` with signal-aware variants
- **Code splitting** — Lazy component loading, chunk management, speculative preloading
- **Server actions** — Type-safe server function calls with in-process transport

### Performance Optimizations

- **Wasm GC proposal support** — Feature detection and type mapping for native GC types (~20-30% binary size reduction)
- **Layout caching** — Taffy layout results cached by component/props/children hashes
- **Text shaping cache** — Cosmic-text shaping results cached by text/font properties
- **GPU resource pooling** — Reusable wgpu buffers, textures, and pipelines
- **Render coalescing** — Frame-aware DOM mutation batching with `requestAnimationFrame`
- **Wasm precompilation** — Wizer/wasmer/wasmtime precompilation for faster cold starts
- **Selective Wasm AOT** — Profile-guided cranelift AOT compilation of hot paths
- **Incremental hydration** — Idle-time batch hydration with priority queue
- **Streaming Wasm compilation** — `StreamingWasmCompiler`, `WasmModuleStreamer`, `IncrementalCompiler`
- **SIMD & threading** — Wasm SIMD intrinsics, multi-threaded Wasm with `SharedArrayBuffer`
- **CSS reactive updates** — Fine-grained CSS property updates without re-render
- **Memory profiler** — `MemoryProfiler` with allocation tracking and leak detection

### Styling & Design

- **Built-in Tailwind** — Tailwind 4.0 (Oxide engine) integration with arbitrary values, container queries, 3D transforms
- **Scoped CSS** — Style encapsulation with shadow DOM
- **CSS-in-Rust** — Reactive style expressions
- **CSS variable theming** — All `rye-ui` components use `var(--rye-*)` custom properties; `ThemeProvider` injects light/dark/auto tokens; runtime theme switching via `data-theme` attribute (no re-render)
- **Design token CLI** — `rpg theme` for creating, exporting, and diffing themes
- **Figma integration** — Design token import and full design-to-code Figma plugin

### Routing & State

- **Type-safe router** — `Router`, `Route`, `RouteMatcher` with params, query, wildcards, nested routes
- **Reactive URL state** — `ReactiveUrlState` for bidirectional signal ↔ URL sync
- **Store pattern** — `Store<T>` with actions, derived selectors, sagas
- **State machines** — Type-safe finite state machines with guard conditions
- **Optimistic updates** — `OptimisticUpdate` with rollback on failure
- **Cross-tab sync** — `CrossTabSync` via `BroadcastChannel` / `LocalStorage` events
- **Offline-first** — `OfflineQueue`, `SyncEngine` with conflict resolution

### Server & Full-Stack

- **SSR** — `SsrRenderer`, `SsrConfig` with streaming, `SsrStream`, partial re-rendering
- **SSG** — Static site generation with `SsgConfig`, `SsgOutput`
- **Server loaders** — `LoaderRegistry`, `LoaderContext`, `LoaderResult` with parallel data loading
- **API routes** — `ApiRoute`, `ApiRouter`, `ApiRequest`, `ApiResponse` with middleware
- **Typed SSE** — Server-sent events with typed event streams
- **Distributed SSR** — `DistributedSsrConfig`, `SsrCache`, `CacheEntry` with edge rendering
- **Database integration** — `rye-db` crate with connection pooling, query helpers, transaction support
- **Islands architecture** — `Island`, `IslandRegistry`, `HydrationStrategy` with selective hydration

### Native & Mobile

- **Native GPU renderer** — wgpu + winit + taffy + cosmic-text for desktop
- **Mobile renderer** — iOS/Android with platform-specific rendering
- **Camera** — Photo/video capture, gallery picking, multi-select
- **Geolocation** — Background tracking, geofencing with entry/exit/dwell
- **Contacts** — Search, field selection, permission management
- **Local notifications** — Time interval, calendar, daily/weekly triggers
- **In-app purchases** — StoreKit/Google Play Billing/Web Payment, consumable/non-consumable/subscription
- **Deep linking** — Universal links, app links, custom URL schemes
- **Background tasks** — Background fetch/processing/sync with constraint-based scheduling
- **Haptic feedback** — Light/medium/heavy/rigid/soft impacts, custom vibration patterns
- **Permissions** — 12 permission types with unified cross-platform API
- **Biometrics** — Face ID/Touch ID/fingerprint authentication
- **Push notifications** — APNs/FCM with topic targeting and silent pushes
- **Lifecycle persistence** — State snapshot save/restore across app kills
- **Widgets** — iOS WidgetKit / Android App Widgets with signal-to-widget binding

### Developer Experience

- **`rpg` CLI** — 21 commands: `new`, `dev`, `build`, `test`, `deploy`, `add`, `upgrade`, `explain`, `scaffold`, `lint`, `doctor`, `playground`, `profile`, `bundle`, `init`, `generate`, `monorepo`, `publish`, `theme`, `docs`, `ci`
- **Hot reloading** — Templates + logic with state preservation
- **Error messages** — Custom diagnostics (R001–R899) with recovery plans, no opaque macro panics
- **AI-aware linter** — `rpg lint --ai` detects common mistakes
- **Playground** — Web-based code editor with live preview and shareable URLs
- **Doctor** — Project health check with auto-fix suggestions
- **Upgrade codemods** — Automatic code transformation during version upgrades
- **Profiler** — `rpg profile` with flamegraph generation
- **Bundle analyzer** — `rpg bundle` with tree map visualization and size reduction suggestions
- **Init wizard** — Interactive project setup with template-based recommendations
- **Code generation** — `rpg generate` from OpenAPI specs and database schemas
- **Editor extensions** — VS Code and JetBrains plugins with full LSP (syntax highlighting, prop autocomplete, signal flow visualization, component preview, error diagnostics)
- **Monorepo support** — `rpg monorepo` for workspace management
- **Component publishing** — `rpg publish` to rye registry with auto-generated docs
- **CI/CD templates** — `rpg ci` generates GitHub Actions, GitLab CI, CircleCI configs

### Testing & Quality

- **Test renderer** — Virtual DOM renderer for unit testing
- **Event simulation** — `EventSimulator` with 20+ event types
- **Query helpers** — Testing Library patterns (`get_by_text`, `get_by_role`, etc.)
- **Integration testing** — Mock SSR server with real HTTP requests and HTML assertions
- **E2E with Playwright** — Multi-browser testing with auto-generated selectors
- **Component contract tests** — Breaking change detection for props, events, slots
- **Performance regression** — Benchmark baselines with render time, bundle size, memory thresholds
- **Semantic snapshot testing** — Structural diffing (elements, props, children) not raw HTML
- **Fuzz testing** — Random template syntax generation, verifies no panics
- **Accessibility tree testing** — `A11yNode` comparison for structural a11y verification
- **Cross-platform equivalence** — Verify identical rendering on web/desktop/mobile
- **Signal update ordering** — Topological sort verification for dependency graphs
- **Trace replay** — Convert runtime traces into regression tests automatically

### AI Integration

- **`rpg explain`** — Error code lookup with text and JSON output
- **MCP server** — 16 tools over Model Context Protocol for AI agents (Claude, Cursor, Windsurf, Copilot)
- **Component discovery** — `ComponentRegistry` with natural language search
- **AI prompt templates** — 10 pre-built templates for common patterns
- **AI code review** — Automated review for common mistakes
- **Context optimization** — Token-budget-aware context packaging
- **Agent SDK** — `rye-mcp` crate for programmatic agent integration
- **AI error codes** — R800–R899 for AI-specific error scenarios

### Ecosystem & Interop

- **React wrapping** — `ReactWrapper` with prop/event mapping and JS bridge code generation
- **Vue wrapping** — `VueWrapper` for Vue SFC mounting with prop/event bridging
- **Tailwind 4.0** — Native Oxide engine integration, zero-config
- **WebGPU compute shaders** — `ComputeShader` with WGSL generation for data-parallel operations
- **Figma plugin** — Full design-to-code export (layout, text, images, interactive states)
- **JS interop** — `wasm-bindgen` integration with typed bindings
- **Web Components** — Custom element wrapping and shadow DOM encapsulation

## Quick Start

```bash
rpg new my-app --template web
cd my-app
rpg dev
```

### Create a component

```bash
rpg add component Counter
rpg add component Dialog --with-props
rpg add component Card --with-props --with-style
```

### The simplest component

```rust
use rye::prelude::*;

#[component]
fn HelloWorld() {
    h1 { "Hello, World!" }
}
```

### Signal-based reactivity

```rust
use rye::prelude::*;

#[component]
fn Counter() {
    let count = Signal::new(0);
    template! {
        <div>
            <p>{count.get()}</p>
            <button on:click=move |_| count.set(count.get() + 1)>{"+1"}</button>
        </div>
    }
}
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `rpg new` | Scaffold a new project |
| `rpg init` | Interactive project wizard |
| `rpg dev` | Start dev server with hot reloading |
| `rpg build` | Build for production (per target) |
| `rpg test` | Run tests (use `--generate` to scaffold) |
| `rpg deploy` | Deploy to web, desktop, or mobile |
| `rpg add` | Add a component, plugin, or package |
| `rpg upgrade` | Upgrade with automatic codemods |
| `rpg explain` | Explain error codes (R001–R899) |
| `rpg scaffold` | Generate component/page/store/action structures |
| `rpg lint` | AI-aware linter (use `--ai` flag) |
| `rpg doctor` | Project health check |
| `rpg playground` | Online code editor with live preview |
| `rpg profile` | Performance profiler with flamegraph |
| `rpg bundle` | Size analyzer with tree map |
| `rpg generate` | Generate code from OpenAPI/DB schema |
| `rpg monorepo` | Workspace management |
| `rpg publish` | Publish component library |
| `rpg theme` | Design token CLI |
| `rpg docs` | Local documentation server |
| `rpg ci` | CI/CD template generator |

## Documentation

- [Manifesto & Philosophy](docs/00-MANIFESTO.md)
- [Competitor Pain Point Audit](docs/01-COMPETITOR-AUDIT.md)
- [Rendering Strategy](docs/02-RENDERING-STRATEGY.md)
- [Template Syntax](docs/03-TEMPLATE-SYNTAX.md)
- [Reactivity Model](docs/04-REACTIVITY-MODEL.md)
- [WASM Optimization](docs/05-WASM-OPTIMIZATION.md)
- [Native Rendering](docs/06-NATIVE-RENDERING.md)
- [Governance](docs/07-GOVERNANCE.md)
- [Developer Ergonomics](docs/08-ERGONOMICS.md)
- [Component Trait](docs/09-COMPONENT-TRAIT.md)
- [Signal Crate](docs/10-SIGNAL-CRATE.md)
- [Template Macro](docs/11-TEMPLATE-MACRO.md)
- [Error Reporting](docs/12-ERROR-REPORTING.md)
- [Event System](docs/13-EVENT-SYSTEM.md)
- [Renderer Trait](docs/14-RENDERER-TRAIT.md)
- [Scheduling & Diffing](docs/15-SCHEDULING-DIFFING.md)
- [Context & DI](docs/16-CONTEXT-DI.md)
- [Async Model](docs/17-ASYNC-MODEL.md)
- [Styling System](docs/18-STYLING-SYSTEM.md)
- [Routing System](docs/19-ROUTING-SYSTEM.md)
- [Form Validation](docs/20-FORM-VALIDATION.md)
- [Animation & Transition](docs/21-ANIMATION-TRANSITION.md)
- [I18N](docs/22-I18N.md)
- [Testing Framework](docs/23-TESTING-FRAMEWORK.md)
- [Positioning](docs/24-POSITIONING.md)
- [Bundler Strategy](docs/25-BUNDLERSTRATEGY.md)
- [Post-V1 Goals (151–250)](docs/26-GOALS.md)
- [AI Optimization](docs/27-AI-OPTIMIZATION.md)
- [Spec for AI](docs/SPEC_FOR_AI.md)
- [V1 Roadmap (150 Goals)](goal.md)
- [Changelog](CHANGELOG.md)

## Workspace Structure

```
rye/
├── crates/
│   ├── rye-core/         # Component, Renderer, Element, Template, hydration, islands, perf, interop (45+ modules)
│   ├── rye-ui/            # Pre-built UI component library — 60+ components, CSS variable theming, light/dark/auto
│   ├── rye-signals/      # Signal, Memo, Effect, Resource, GlobalSignal, Store, state machines (17 modules)
│   ├── rye-macros/       # template! macro, #[component] attribute
│   ├── rye-html/         # DOM renderer (web-sys, WASM)
│   ├── rye-router/       # Type-safe routing with params, query, nested routes
│   ├── rye-forms/        # Reactive forms + validation
│   ├── rye-i18n/         # Internationalization with compile-time message extraction
│   ├── rye-animations/   # Transitions, spring physics
│   ├── rye-ssr/          # SSR, streaming, SSG, server loaders, API routes, typed SSE (21 modules)
│   ├── rye-desktop/      # Native GPU renderer (wgpu + winit + taffy + cosmic-text)
│   ├── rye-mobile/       # iOS/Android: camera, geolocation, contacts, IAP, push, widgets (18 modules)
│   ├── rye-testing/      # Test renderer, integration harness, E2E, contract, fuzz, a11y, trace replay
│   ├── rye-devtools/     # Component inspector, profiler
│   ├── rye-cli/          # 21 CLI commands (rpg)
│   ├── rye-serialize/    # Minimal serializer for SSR state transfer
│   ├── rye-mcp/          # MCP server — 16 tools for AI agent integration
│   └── rye-db/           # Database integration with connection pooling
├── docs/                 # 28 design documents + AI spec
├── goal.md               # V1 roadmap (150 goals)
├── CHANGELOG.md          # Full changelog (250 goals)
├── Cargo.toml            # Workspace root (17 crates)
└── README.md
```

## Key Differentiators

| Pain | Solved By |
|---|---|
| React's hooks rules | Signals — create anywhere, no rules |
| React's manual optimization | Auto-tracking reactivity — no `useMemo`/`useCallback` |
| React's no built-in state | `Store`, `GlobalSignal`, state machines — all built-in |
| React Native separate codebase | One codebase, all platforms |
| Vue's dual API confusion | One API: signals + components, no Options API |
| Vue's no cross-platform | Web + desktop (native GPU) + mobile from one codebase |
| Dioxus's WebView-only desktop | Native GPU renderer via `wgpu` |
| Dioxus's WASM size | Aggressive tree-shaking, <50KB target, WasmGC support |
| Dioxus's bad error messages | Custom diagnostic layer with friendly errors (R001–R899) |
| Dioxus's weak CSS | Built-in Tailwind 4.0, scoped CSS, CSS-in-Rust, reactive styles |
| Leptos's web-only | Full cross-platform from day one |
| Yew's stale patterns | Modern signal-based reactivity, no class components |
| Rust's steep curve | Great docs, interactive tutorial, helpful errors, zero-config CLI |
| All frameworks' testing pain | 13 testing modules: unit, integration, E2E, contract, fuzz, a11y, perf regression |
| All frameworks' i18n afterthought | Compile-time message extraction, built-in |
| All frameworks' a11y afterthought | Semantic tree, screen reader support, a11y tree testing, built-in |
| No AI-friendly framework | MCP server, `rpg explain`, AI linter, prompt templates, agent SDK |
| React/Vue migration lock-in | `wrap_react_component!()`, `wrap_vue_component!()` for incremental migration |

## License

Dual-licensed under MIT or Apache-2.0.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [Governance](docs/07-GOVERNANCE.md).
