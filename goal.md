# 100 Goals to V1/MVP — The Ultimate Rust UI Framework

> **Status disclaimer (August 2026):** The `[x]` markers below indicate that
> code exists for each goal, NOT that the goal is fully implemented or proven.
> Many goals have scaffold/stub implementations only. A re-audit is in progress
> to reclassify each goal as `[built]` (compiles + tested), `[stub]` (types
> defined, no real implementation), or `[untested]` (compiles but no test
> coverage). The most significant gaps:
>
> - **`rye-mobile`** — 6,500 LOC, zero FFI bindings. All mobile goals are
>   effectively `[stub]`.
> - **`template!` macro** — parses basic syntax but the demo app avoids it,
>   meaning it has never been stress-tested against real UI code.
> - **No deployed app** — no goal involving "deploy" or "production" has been
>   verified end-to-end.
>
> The `[x]` markers are preserved for traceability but should not be read as
> "done."

## Phase 1: Research & Foundation (Goals 1–10)

1. [x] **Define the framework's core philosophy** — Write a manifesto: "Rust safety + React ergonomics + Vue simplicity + SolidJS performance + Svelte DX." Zero-config, batteries-included, cross-platform, developer-happiness-first. → `docs/00-MANIFESTO.md`

2. [x] **Audit all competitor pain points** — Catalog every known pain from React (hooks rules, prop drilling, re-render storms), Vue (dual API confusion, SFC lock-in), Dioxus (WASM size, WebView-only desktop, weak CSS), Leptos (docs, web-only), Yew (stale patterns), Svelte (compiler lock-in), Angular (bloat), Solid (ecosystem size). → `docs/01-COMPETITOR-AUDIT.md`

3. [x] **Choose the rendering strategy** — Hybrid compile-time + fine-grained signals. → `docs/02-RENDERING-STRATEGY.md`

4. [x] **Choose the template syntax** — HTML-like `template!` macro with Rust expressions. → `docs/03-TEMPLATE-SYNTAX.md`

5. [x] **Define the reactivity model** — Signal-based with automatic dependency tracking, no rules of hooks, batched updates, Send+Sync. → `docs/04-REACTIVITY-MODEL.md`

6. [x] **Research WASM binary size optimization strategies** — 10 strategies documented, <50KB target set. → `docs/05-WASM-OPTIMIZATION.md`

7. [x] **Research native rendering paths** — Decided on `wgpu` + `winit` + `taffy` + `cosmic-text` for native GPU rendering. → `docs/06-NATIVE-RENDERING.md`

8. [x] **Establish project governance** — Roles, RFC process, CoC, contributing guidelines, release process, LTS policy. → `docs/07-GOVERNANCE.md`, `CONTRIBUTING.md`

9. [x] **Set up the monorepo structure** — Cargo workspace with 15 crates: `rye-core`, `rye-signals`, `rye-macros`, `rye-html`, `rye-router`, `rye-forms`, `rye-i18n`, `rye-animations`, `rye-ssr`, `rye-desktop`, `rye-mobile`, `rye-testing`, `rye-devtools`, `rye-cli`, `rye-serialize`. → `Cargo.toml`, `crates/`

10. [x] **Bootstrap CI/CD pipeline** — 4 workflows: ci.yml (fmt, clippy, test on Linux/macOS/Windows, WASM build, audit, docs, MSRV), size-check.yml, benchmarks.yml, release.yml. Plus dependabot.yml. → `.github/workflows/`

---

## Phase 2: Core Architecture & Design (Goals 11–25)

11. [x] **Design the component trait system** — Function + trait components, lifecycle (create/render/destroy), Props derive. → `docs/09-COMPONENT-TRAIT.md`

12. [x] **Design the signal/primitive reactivity crate** — Signal, Memo, Effect, Resource with auto-tracking, no hook rules, batched updates, Read/Write split, SyncSignal. → `docs/10-SIGNAL-CRATE.md`

13. [x] **Design the template macro (`template!`)** — HTML-like syntax, compile-time optimization, components/conditionals/loops/fragments, custom errors. → `docs/11-TEMPLATE-MACRO.md`

14. [x] **Design custom error reporting** — Custom diagnostics with error codes, source mapping, Levenshtein suggestions, multi-line errors. → `docs/12-ERROR-REPORTING.md`

15. [x] **Design the event system** — Typed events (Rust enums), event delegation at root, cross-platform mapping, custom events. → `docs/13-EVENT-SYSTEM.md`

16. [x] **Design the rendering abstraction (Renderer trait)** — Renderer trait with typed nodes, batch operations, 4 backends (DOM, Native, SSR, Test). → `docs/14-RENDERER-TRAIT.md`

17. [x] **Design the scheduling/diffing engine** — Dual-mode (signal default, VDOM optional, auto-detect), keyed reconciliation, update priority, batched scheduling. → `docs/15-SCHEDULING-DIFFING.md`

18. [x] **Design context/dependency injection** — Type-safe reactive context, provide/consume, subtree override, no external library. → `docs/16-CONTEXT-DI.md`

19. [x] **Design the async model** — Resource with auto-cancellation, Suspense boundaries, ErrorBoundary, streaming SSR, resource caching. → `docs/17-ASYNC-MODEL.md`

20. [x] **Design the styling system** — 4 approaches: scoped CSS (style!), CSS-in-Rust (css!), built-in Tailwind, reactive CSS variables. → `docs/18-STYLING-SYSTEM.md`

21. [x] **Design the routing system** — File-based + code-based, typed params, route guards, lazy loading, type-safe links, SSR-aware. → `docs/19-ROUTING-SYSTEM.md`

22. [x] **Design the form & validation system** — Reactive forms, derive(Form), built-in validators, async validation, field state (touched/dirty), error helpers. → `docs/20-FORM-VALIDATION.md`

23. [x] **Design the animation/transition system** — Transition, TransitionGroup, FLIP for lists, spring physics, CSS integration, native (wgpu) animations. → `docs/21-ANIMATION-TRANSITION.md`

24. [x] **Design internationalization (i18n)** — Compile-time extraction, reactive locale, Fluent/ICU MessageFormat, lazy-loaded translations, date/number formatting, SSR-aware. → `docs/22-I18N.md`

25. [x] **Design the testing framework** — TestRenderer, query helpers, event simulation, signal testing, snapshot testing, SSR testing, Playwright E2E. → `docs/23-TESTING-FRAMEWORK.md`

---

## Phase 3: Core Implementation — Reactivity & Rendering (Goals 26–40)

26. **Implement the signal crate (`rye-signals`)** — `Signal<T>` with automatic dependency tracking, batched updates, `Read`/`Write` split for borrow-checker friendliness, `SyncSignal<T>` for `Send + Sync` multi-threaded runtimes. → Design: `docs/10-SIGNAL-CRATE.md`

27. **Implement global signals** — `GlobalSignal<T>` for app-wide state without context boilerplate. Thread-safe (`Arc<Mutex>`), reactive, with devtools integration hooks. → Design: `docs/10-SIGNAL-CRATE.md`

28. **Implement derived/computed state** — `Memo<T>` that automatically re-computes when dependencies change. Support for chaining memos. No manual dependency arrays. `use_memo()` hook. → Design: `docs/10-SIGNAL-CRATE.md`

29. **Implement effect system** — `Effect` for side effects with automatic cleanup. Support for `use_effect()`, `on_cleanup()`, scoped effects tied to component lifecycle. → Design: `docs/10-SIGNAL-CRATE.md`

30. **Implement the `Renderer` trait** — Abstract interface: `create_element`, `create_text`, `set_text`, `set_attribute`, `remove_attribute`, `insert_child`, `remove_child`, `replace_child`, `move_child`, `set_event_listener`, `remove_event_listener`. Optional `BatchRenderer` for batch operations. Every backend implements this. → Design: `docs/14-RENDERER-TRAIT.md`

31. **Implement the WASM/DOM renderer (`rye-html`)** — First renderer target. Uses `web-sys` for DOM manipulation. Implements `BatchRenderer` for batched writes. Optimize: `DocumentFragment` for bulk inserts, minimize reflows, minimal `web-sys` feature flags. → Design: `docs/14-RENDERER-TRAIT.md`, `docs/05-WASM-OPTIMIZATION.md`

32. **Implement the SSR renderer (`rye-ssr`)** — String-based renderer for server-side rendering. Streaming support via async streams. Hydration markers (`data-rye-*`) embedded in HTML. `render_to_string()` and `render_to_stream()` APIs. → Design: `docs/14-RENDERER-TRAIT.md`, `docs/17-ASYNC-MODEL.md`

33. **Implement the virtual/test renderer (`rye-testing`)** — In-memory `TestRenderer` for testing. No browser/WASM needed. Enables fast unit tests in CI. Includes query helpers (`get_by_tag`, `get_by_test_id`, `get_by_text`) and event simulation. → Design: `docs/14-RENDERER-TRAIT.md`, `docs/23-TESTING-FRAMEWORK.md`

34. **Implement the template macro (`rye-macros`)** — `template!` macro that parses HTML-like syntax at compile time, generates optimized node creation code with static/dynamic separation. Supports implicit `template!` inside `#[component]`, shorthand event handlers (`onclick: count += 1`), components, conditionals, loops, fragments. Custom error diagnostics with error codes (R001–R799). → Design: `docs/11-TEMPLATE-MACRO.md`, `docs/12-ERROR-REPORTING.md`, `docs/08-ERGONOMICS.md`

35. **Implement component function support (`rye-macros`)** — `#[component]` macro that wraps a Rust function into a component with typed props (`#[derive(Props)]`), default values (`#[prop(default)]`), optional props (`#[prop(optional)]`), children (`#[prop(children)]`). Auto-return type inference (drop `-> Element`). Implicit `template!` wrapping. → Design: `docs/09-COMPONENT-TRAIT.md`, `docs/08-ERGONOMICS.md`

36. **Implement keyed list reconciliation** — Efficient O(n) diffing for lists with keys via `For` component. Handle insertions, deletions, moves without full re-render. Benchmark against React and Solid. → Design: `docs/15-SCHEDULING-DIFFING.md`

37. **Implement event delegation** — Attach one listener per event type at root, dispatch via `HandlerRegistry` (like SolidJS). Reduces memory and improves performance. Typed events (`MouseEvent`, `KeyboardEvent`, `InputEvent`, etc.) with `prevent_default()` and `stop_propagation()`. → Design: `docs/13-EVENT-SYSTEM.md`

38. **Implement context system** — `provide_context<T>()`, `provide_context_signal()`, `use_context<T>()`, `use_context_signal<T>()` with type-safe injection via `TypeId` map. Reactive context updates. Subtree override support. → Design: `docs/16-CONTEXT-DI.md`

39. **Implement suspense & error boundaries** — `Suspense {}` component with fallback for loading states, `ErrorBoundary {}` with render-prop for error UI. `Resource<T>` with auto-cancellation and `ResourceState` (Pending/Ready/Error). Async-aware rendering pipeline. → Design: `docs/17-ASYNC-MODEL.md`

40. **Implement hydration (`rye-html`)** — After SSR, client-side WASM "hydrates" the server-rendered HTML by reading `data-rye-*` markers, attaching event listeners and signal subscriptions without re-rendering. Critical for performance + SEO. → Design: `docs/17-ASYNC-MODEL.md`, `docs/14-RENDERER-TRAIT.md`

---

## Phase 4: Component Model & DX (Goals 41–55)

41. **Implement props system** — Typed props with:
    - Required and optional props
    - Default values
    - Spread props (`{...props}`)
    - Children as typed slot
    - Compile-time validation (missing required prop = compile error)

42. **Implement slots & named children** — Beyond simple children: named slots (like Vue), scoped slots, fallback content. `slot: Header` syntax in props.

43. **Implement dynamic components** — `<Component {dyn_component} />` for rendering arbitrary component types at runtime. Needed for routing, plugin systems, dynamic UIs.

44. **Implement component lifecycle hooks** — `on_create`, `on_mount`, `on_update`, `on_destroy`, `on_cleanup`. All optional, all ergonomic.

45. **Implement `use_resource` / async data** — Async primitive that:
    - Automatically tracks signal dependencies
    - Cancels on unmount or dependency change
    - Returns `Pending | Ready(T) | Error(E)` state
    - Integrates with `<Suspense>`

46. **Implement refs** — `use_ref` for direct DOM/native node access. Type-safe, platform-abstracted via `ElementRef` trait.

47. **Implement forward_ref pattern** — Allow components to expose refs to inner elements (like `React.forwardRef`). Needed for building component libraries.

48. **Implement portals** — Render children into a different part of the tree (modals, tooltips, popovers). Cross-platform: DOM portal, native window, etc.

49. **Implement `<Show>` and `<For>` control components** — Declarative conditional and list rendering that integrate with signals (no manual re-render triggers).

50. **Implement lazy components** — `lazy(|| import!("./HeavyComponent"))` for code splitting. Works with WASM dynamic imports and native dynamic loading.

51. **Implement error recovery** — When a component panics, catch it, render error boundary, allow retry. Never crash the entire app.

52. **Implement component memoization (automatic)** — Components are memoized by default based on prop equality. No manual `React.memo`. Override with `#[component(memo = false)]`.

53. **Implement `use_callback` alternative** — Since signals track automatically, callbacks don't need memoization. But provide `use_callback` for interop with non-reactive APIs.

54. **Implement teleport/portal for SSR** — During SSR, portals render inline with markers. On hydration, they move to correct position. Handles modals in SSR correctly.

55. **Implement strict mode** — Development-only mode that:
    - Double-invokes effects to catch bugs
    - Warns about memory leaks
    - Detects signal write-during-render
    - Validates prop types at runtime

---

## Phase 5: Rendering Targets & Cross-Platform (Goals 56–70)

56. **Implement the native GPU renderer (desktop)** — Using `wgpu` + `winit`, render UI directly to GPU. No WebView. True native performance. Text via `cosmic-text` or `swash`.

57. **Implement layout engine for native renderer** — Taffy (flexbox/grid layout engine in Rust) for native layout. Same layout semantics as CSS flexbox/grid.

58. **Implement the desktop window manager** — Window creation, multi-window support, system tray, native menus, file dialogs. Using `tao` or custom `winit` wrapper.

59. **Implement mobile renderer (iOS/Android)** — Either GPU renderer (same as desktop, scaled) or platform-native (UIKit/Compose) bridge. Start with GPU renderer for simplicity.

60. **Implement mobile lifecycle integration** — Handle app background/foreground, memory warnings, orientation changes, safe area insets.

61. **Implement cross-platform event abstraction** — Unified event types: `ClickEvent`, `InputEvent`, `KeyEvent`, `FocusEvent`, `ResizeEvent`, `TouchEvent`, `ScrollEvent` — same API on web, desktop, mobile.

62. **Implement platform capability detection** — `#[cfg(target = "web")]`, runtime `Platform::is_web()`, conditional rendering based on platform. `use_platform()` hook.

63. **Implement shared element transitions** — Cross-platform animated transitions between views (like Hero animations in Flutter). Works on web (FLIP) and native (GPU).

64. **Implement responsive design primitives** — `use_media_query()`, `<Responsive>`, breakpoint system. Same API across platforms.

65. **Implement accessibility (a11y) layer** — Semantic tree generation, ARIA support on web, accessibility API on native (UIAccessibility on iOS, AccessibilityNodeProvider on Windows). Screen reader support out of the box.

66. **Implement keyboard navigation** — Focus management, tab order, keyboard shortcuts, focus trap (for modals). Cross-platform.

67. **Implement clipboard & drag-and-drop** — Cross-platform clipboard access, native drag-and-drop on desktop, HTML5 DnD on web.

68. **Implement file system access** — Cross-platform file read/write with permissions handling. Web: File System Access API. Desktop: native FS. Mobile: platform storage.

69. **Implement networking abstraction** — `use_fetch` hook with:
    - Request cancellation
    - Caching
    - Retry logic
    - SSR support (fetch on server, hydrate on client)
    - Type-safe request/response with serde

70. **Implement WebView fallback renderer** — For platforms where GPU renderer isn't available or for hybrid apps. Uses the same component tree, renders to WebView.

---

## Phase 6: Built-in Features & Batteries (Goals 71–85)

71. **Implement the router crate** — Full-featured router with:
    - Nested routes
    - Typed route params (`/users/:id` → `id: String`)
    - Query params
    - Route guards
    - Lazy loading
    - History API (web) + native navigation (desktop/mobile)
    - `<Link>` component with active state

72. **Implement the styling engine** — Compile-time CSS:
    - `style!` macro for scoped styles
    - CSS-in-Rust with type checking (`color: Color::RED`)
    - Tailwind utility class support (built-in, zero config)
    - CSS variables with reactive signal bindings
    - Auto-prefixing for web target
    - Critical CSS extraction for SSR

73. **Implement the forms crate** — Reactive forms:
    - `use_form()` with field tracking
    - Validation (sync + async)
    - Dirty/pristine/touched state
    - Submit handling
    - Error display components
    - Schema integration (`serde` + `validator`)

74. **Implement the i18n crate** — Internationalization:
    - Compile-time message extraction from `template!` macros
    - `.json` / `.toml` / `.yaml` translation files
    - Reactive locale switching
    - Pluralization (CLDR rules)
    - Date/number/currency formatting
    - Lazy-loaded locale bundles

75. **Implement the animation crate** — Declarative animations:
    - `<Transition>` (enter/leave)
    - `<TransitionGroup>` (list animations)
    - Spring physics (`Spring::new(0.0).to(100.0)`)
    - Gesture-driven animations (drag, swipe, pinch)
    - Shared element transitions
    - Timeline/keyframe API

76. **Implement the state management crate** — Beyond signals:
    - `Store<T>` for complex state (like Pinia/Zustand)
    - State machines (XState-like, type-safe)
    - Time-travel debugging support
    - State persistence (localStorage/IndexedDB/file)
    - Devtools serialization

77. **Implement headless UI primitives** — Unstyled, accessible components:
    - `Dialog`/`Modal`, `Popover`, `Tooltip`, `Menu`, `Tabs`, `Accordion`, `Combobox`, `Select`, `Switch`, `Slider`, `DatePicker`
    - All accessible, all keyboard-navigable, all cross-platform

78. **Implement the HTTP/data-fetching crate** — Type-safe data layer:
    - `use_query()` / `use_mutation()` (like TanStack Query)
    - Automatic caching, refetching, invalidation
    - Optimistic updates
    - Request deduplication
    - SSR prefetching + hydration
    - WebSocket / SSE integration

79. **Implement code splitting & lazy loading** —
    - Route-level splitting (automatic with router)
    - Component-level splitting (`lazy!` macro)
    - WASM: dynamic imports via `wasm-bindgen`
    - Native: dynamic library loading
    - Shared chunk optimization

80. **Implement the headless SSR/SSG/ISR system** —
    - `render_to_string()` for SSR
    - `generate_static_pages()` for SSG
    - Incremental static regeneration (ISR)
    - Streaming SSR with `<Suspense>`
    - Hydration with zero re-render
    - SEO meta management (`<head>` management)

81. **Implement the plugin/extension system** —
    - Plugin trait: `fn install(&self, app: &mut App)`
    - Lifecycle hooks: `before_render`, `after_render`, `on_error`
    - Middleware pattern for state, routing, fetching
    - Official plugins: devtools, analytics, error reporting, PWA

82. **Implement the CLI tool** — `framework-cli`:
    - `new` — scaffold project (web, desktop, mobile, fullstack)
    - `dev` — dev server with hot reloading
    - `build` — production build (per target)
    - `test` — run tests
    - `deploy` — deploy to web (static host), desktop (installer), mobile (APK/IPA)
    - `add` — add component/plugin from registry
    - `upgrade` — framework upgrade with codemods

83. **Implement hot reloading** —
    - Template/CSS hot reload (preserve state)
    - Signal/logic hot reload (reset state with warning)
    - Works on web (WASM) and desktop (native)
    - File watcher with debouncing
    - Error overlay in browser/app

84. **Implement the devtools protocol** —
    - Component tree inspector
    - Signal/state viewer with time-travel
    - Event log
    - Performance profiler (flame charts)
    - Render highlight (show what re-rendered)
    - Memory usage tracker
    - Browser extension + standalone desktop app

85. **Implement the testing utilities crate** —
    - `render!(component)` → virtual DOM for testing
    - `fire_event(element, event)` — simulate events
    - `screen::get_by_text()`, `get_by_role()` — query helpers (like Testing Library)
    - `assert_rendered!(component, expected_html)`
    - Snapshot testing
    - Async testing support
    - Mock signals/resources

---

## Phase 7: Developer Experience & Polish (Goals 86–95)

86. **Implement world-class compiler diagnostics** — Every macro error produces:
    - Colored, contextual output
    - Error code + link to docs
    - Suggested fix (like Rust's `help:` messages)
    - No opaque `proc-macro` panics
    - Custom `#[diagnostic]` attribute system

87. **Implement IDE support** —
    - LSP server for the framework
    - Syntax highlighting for `template!` macro (Tree-sitter grammar)
    - Autocompletion for components, props, events
    - Go-to-definition for components and signals
    - Inline type hints in templates
    - VS Code extension (first-class)

88. **Implement the project scaffolding templates** —
    - `web-app` — SPA with router
    - `ssr-app` — fullstack with SSR
    - `desktop-app` — native desktop
    - `mobile-app` — iOS/Android
    - `fullstack` — web + desktop + mobile from one codebase
    - `component-library` — for building reusable component packages
    - `plugin` — for framework plugins

89. **Implement the package/component registry** —
    - `framework add @ui/button` — install from registry
    - Registry website with search, docs, demos
    - Versioned components
    - Component playground (live edit in browser)
    - Theme system for registry components

90. **Implement the theming system** —
    - Design tokens (colors, spacing, typography, shadows)
    - Dark/light mode with reactive switching
    - Theme provider with signal-based overrides
    - CSS variable generation
    - Theme presets (Material, Tailwind, custom)
    - Runtime theme switching

91. **Implement PWA support** —
    - Service worker generation
    - Web manifest
    - Offline support
    - Install prompt
    - Push notifications (web + native)
    - Background sync

92. **Implement security best practices** —
    - XSS prevention (auto-escaping in templates, `raw_html` must be explicit)
    - CSRF token integration
    - Content Security Policy helpers
    - Subresource integrity
    - No `eval` / no dynamic code execution
    - Memory safety (Rust's guarantee)

93. **Implement performance monitoring** —
    - Built-in performance marks
    - Component render time tracking
    - Signal update frequency tracking
    - Bundle size analyzer
    - Lighthouse integration (web)
    - Memory leak detection

94. **Implement migration tooling** —
    - React → our framework codemod (AST transformation)
    - Vue SFC → our framework codemod
    - Dioxus → our framework codemod
    - HTML → `template!` macro converter
    - CSS → scoped style converter
    - Reduces adoption friction massively

95. **Implement the documentation site** —
    - Interactive tutorial (like Rustlings but for the framework)
    - API reference (auto-generated from rustdoc)
    - Guides: Getting started, Routing, State, SSR, Desktop, Mobile, Testing, Deployment
    - Examples gallery (live, editable)
    - Comparison pages (vs React, vs Vue, vs Dioxus, vs Leptos)
    - Architecture deep dives
    - Video tutorials

---

## Phase 8: Ecosystem, Community & V1 Release (Goals 96–100)

96. **Build the example app showcase** —
    - TodoMVC (benchmark baseline)
    - Realworld app (full Medium clone — Conduit)
    - Hacker News clone
    - E-commerce demo (with cart, checkout, SSR)
    - Desktop app demo (with native menus, system tray)
    - Mobile app demo (with gestures, transitions)
    - All from the same codebase with platform-specific tweaks

97. **Establish the community infrastructure** —
    - Discord server (help, showcase, dev channels)
    - GitHub Discussions for Q&A
    - RFC repository for proposals
    - Contributing guide + "good first issue" labels
    - Office hours / community calls
    - Conference talk submissions (RustConf, JSConf, etc.)
    - Blog posts and technical write-ups

98. **Run the beta program** —
    - Release `0.1.0-beta` to crates.io
    - Recruit 10-20 early adopters for real-world testing
    - Collect feedback on DX, performance, pain points
    - Fix critical bugs, polish rough edges
    - Publish "what we learned" blog posts
    - Iterate to `0.2.0-beta`, `0.3.0-beta`

99. **Achieve benchmark parity or superiority** —
    - js-framework-benchmark: match or beat SolidJS (the current leader)
    - WASM bundle size: <50KB gzipped for hello world
    - SSR throughput: >100k requests/sec on single core
    - Memory usage: <2x vanilla JS
    - Cold start (desktop): <100ms
    - Publish all benchmarks publicly with reproducible setups

100. **Ship V1.0.0** —
    - Semantic versioning commitment (no breaking changes in 1.x)
    - LTS support policy (2 years minimum)
    - Migration guide from beta
    - "Stable" badge on all major features
    - Launch blog post + demo video
    - crates.io publish
    - npm publish (for WASM bindings/wrapper)
    - Homebrew/Scoop/winget package for CLI
    - Documentation site live
    - Example apps deployed
    - **Celebrate. Then start planning V2.**

---

## Summary: The Key Differentiators

| Pain | Solved By |
|---|---|
| React's hooks rules | Signals — create anywhere, no rules |
| React's manual optimization | Auto-tracking reactivity — no `useMemo`/`useCallback` |
| React's no built-in state | `Store`, `Signal::global`, state machines — all built-in |
| React Native separate codebase | One codebase, all platforms |
| Vue's dual API confusion | One API: signals + components, no Options API |
| Vue's no cross-platform | Web + desktop (native GPU) + mobile from one codebase |
| Dioxus's WebView-only desktop | Native GPU renderer via `wgpu` |
| Dioxus's WASM size | Aggressive tree-shaking, <50KB target |
| Dioxus's bad error messages | Custom diagnostic layer with friendly errors |
| Dioxus's weak CSS | Built-in Tailwind, scoped CSS, CSS-in-Rust, reactive styles |
| Leptos's web-only | Full cross-platform from day one |
| Yew's stale patterns | Modern signal-based reactivity, no class components |
| Rust's steep curve | Great docs, interactive tutorial, helpful errors, zero-config CLI |
| All frameworks' testing pain | Built-in testing framework, Testing Library patterns |
| All frameworks' i18n afterthought | Compile-time message extraction, built-in |
| All frameworks' a11y afterthought | Semantic tree, screen reader support, built-in |
