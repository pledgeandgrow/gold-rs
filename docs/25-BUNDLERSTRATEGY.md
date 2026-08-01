# Pledgepack × rye — Universal Bundler Strategy

> One bundler. Every platform. First cross-compile bundler.

---

## Vision

No existing bundler handles all targets. Pledgepack is the first:

| Tool | Web | RN | Native | Rust/Wasm |
|------|-----|-----|--------|-----------|
| Metro | No | Yes | No | No |
| Vite | Yes | No | No | No |
| Webpack | Yes | Partial | No | No |
| Turbopack | Yes | No | No | No |
| esbuild | Yes | No | No | No |
| Parcel | Yes | No | No | No |
| Trunk | No | No | No | Yes (Rust only) |
| **Pledgepack** | **Yes** | **Implemented** | **Implemented** | **Implemented** |

---

## The Pitch

```bash
pledge web dev          # Web dev server (Vite replacement)
pledge rn dev           # React Native dev server (Metro replacement)
pledge rye dev          # rye Wasm dev server
pledge rye build ios    # rye native iOS app
pledge rye build apk    # rye native Android app
pledge web build        # Production web bundle
```

One bundler for every platform. No other tool does this.

---

## Architecture

### Shared infrastructure (Zig hot paths — all adapters)

```
Shared (all adapters):
  ├── io.zig (io_uring file reads)     ← always fast
  ├── graph.zig (arena module graph)   ← always fast
  └── simd.zig (source scanning)       ← always fast
```

These modules from pledgepack work unchanged for any target. File I/O and module graph traversal are the same regardless of output format.

### Per-adapter (only loaded when invoked)

```
Per-adapter (only loaded when used):
  ├── adapter-web: JS bundling, CSS, HTML
  ├── adapter-rn: Hermes, platform resolution, Metro protocol
  ├── adapter-rye: wasm-pack orchestration, hydration
  └── adapter-rye-native: cross-compile, wgpu
```

---

## Adapter: rye (web)

### What it does

```
pledgepack-cli (dev)
  ├── Watch .rs files
  ├── On change:
  │   ├── Run cargo check (fast, incremental)
  │   ├── Run wasm-pack build --dev (if .rs changed)
  │   ├── Transform output JS glue
  │   └── Push HMR update to browser
  ├── Serve:
  │   ├── HTML shell (with hydration markers)
  │   ├── .wasm binary
  │   ├── JS glue (wasm-bindgen output)
  │   └── CSS / assets
  └── WebSocket → HMR channel
```

### HTML template

```html
<!DOCTYPE html>
<html>
<head>
  <link rel="preload" href="/app.wasm" as="fetch" crossorigin>
</head>
<body>
  <div id="app"><!--rye-0-e--></div>
  <script type="module">
    import init from "/app.js";
    init("/app.wasm").then(() => {
      // rye runtime hooks into HMR via WebSocket
    });
  </script>
  <script src="/hmr-client.js"></script>
</body>
</html>
```

### HMR approaches

**Approach A: Full re-instantiate (simple, ~1-3s)**
```
.rs file changes
  → cargo recompile (incremental, ~0.5-2s)
  → wasm-pack build --dev (~0.5s)
  → WebSocket message: { type: "wasm-reload", url: "/app.wasm?v=2" }
  → Browser: re-instantiate Wasm, re-run effects, preserve signal state
```

**Approach B: Template-only hot swap (fast, <100ms)**
```
.rs file changes but only template! content changed
  → Parse template! macro output
  → Diff against previous template
  → WebSocket message: { type: "template-patch", ops: [...] }
  → Browser: apply DOM operations directly, no Wasm recompile
```

Both approaches are implemented. Approach A (full re-instantiate) is the default; Approach B (template-only hot swap) is used when only `template!` content changes.

### Implementation

```rust
// crates/adapter-rye/src/lib.rs
pub struct RyeAdapter {
    wasm_out_dir: PathBuf,
    html_template: String,
}

impl Adapter for RyeAdapter {
    fn name(&self) -> &str { "rye" }

    fn matches(&self, path: &str) -> bool {
        path.ends_with(".rs") || path.ends_with(".wasm")
    }

    fn transform(&self, path: &str, content: &str) -> Result<TransformOutput> {
        // For .rs files: trigger wasm-pack build
        // For .wasm files: copy to serve dir
        // For HTML: inject hydration script + wasm loader
    }

    fn resolve(&self, import: &str, from: &str) -> Option<String> {
        // Resolve rye-specific imports
        // e.g. "rye:html" → crates/rye-html/src/lib.rs
    }
}
```

### Commands

```bash
pledge rye dev    # Start dev server with rye adapter
pledge rye build  # Production build (wasm-opt -Oz, tree-shake)
pledge rye serve  # Serve production build locally
```

---

## Adapter: React Native (Metro replacement)

### What it does

```
pledgepack-cli (dev/build)
  ├── adapter-react-native/
  │   ├── Resolve .ios.js / .android.js / .native.js extensions
  │   ├── Strip web-only imports (DOM, CSS, web APIs)
  │   ├── Inject RN polyfills (StyleSheet, View, Text)
  │   ├── Transform JSX → RN-compatible createElement
  │   ├── Bundle → single JS file or Hermes bytecode
  │   ├── Serve via WebSocket → Metro-compatible protocol
  │   └── Connect to RN runtime on device/simulator
```

### Metro protocol compatibility

React Native's dev server speaks a specific WebSocket protocol. The RN client on the device expects:
- Bundle requests at `/index.bundle?platform=ios&dev=true`
- HMR updates via WebSocket with specific message format
- Source map requests at `/index.map`

Pledgepack's axum server would need to implement this protocol. Metro's protocol is documented but has quirks — reading Metro's source code may be necessary.

### Hermes bytecode

In production, RN compiles JS to Hermes bytecode (`.hbc` files) for faster startup. Options:
- Ship the Hermes compiler as a subprocess (simplest)
- Link Hermes as a C++ dependency (faster, more complex)

### Platform-specific resolution

```
import { View } from 'react-native';
// On web: polyfill or error
// On iOS: RN's View component
// On Android: RN's View component (different implementation)
```

Pledgepack's resolver needs platform-aware logic: `.ios.js`, `.android.js`, `.native.js`, `.web.js` extension resolution.

### Native module bridging

RN apps call native modules (Camera, Geolocation, etc.) via a bridge. The bundler doesn't compile these — but it needs to know which native modules exist to generate the bridge registration code.

### Commands

```bash
pledge rn dev --platform ios      # Dev server for iOS simulator
pledge rn dev --platform android  # Dev server for Android emulator
pledge rn build --platform ios --release  # Production Hermes bytecode
```

---

## Adapter: rye native (mobile/desktop)

### What it does

```
pledge rye build ios
  ├── cargo build --target aarch64-apple-ios
  ├── wgpu + winit → native GPU rendering
  ├── No JS, no Wasm, no React Native
  └── Output: .app bundle for iOS

pledge rye build apk
  ├── cargo build --target aarch64-linux-android
  ├── wgpu + winit → native GPU rendering
  ├── No JS, no Wasm, no React Native
  └── Output: .apk for Android
```

This is the real cross-compile play — pure Rust on mobile, no JS runtime at all.

---

## Feature flags — Zero bloat

Each adapter is a Cargo feature. Only compiled when needed:

```toml
# pledgepack-cli/Cargo.toml
[features]
default = ["adapter-web"]
adapter-web = ["pledgepack-adapter-web"]
adapter-react-native = ["pledgepack-adapter-react-native"]
adapter-rye = ["pledgepack-adapter-rye"]
adapter-rye-native = ["pledgepack-adapter-rye-native"]
all = ["adapter-web", "adapter-react-native", "adapter-rye", "adapter-rye-native"]
```

```bash
# Web-only build (small):
cargo build -p pledgepack-cli --no-default-features --features adapter-web

# Everything build (larger, but still one binary):
cargo build -p pledgepack-cli --features all
```

---

## Performance & bundle size impact

### Zero impact on output bundles

| Command | Output contains | Output size |
|---------|----------------|-------------|
| `pledge web build` | JS chunks, CSS, assets | Same as now |
| `pledge rn build` | JS bundle (Hermes bytecode) | No web code, no Wasm |
| `pledge rye build` | .wasm + JS glue + HTML | No RN polyfills |
| `pledge rye build ios` | Native binary (wgpu) | No JS at all |

No cross-contamination. A web bundle never includes RN polyfills. A RN bundle never includes Wasm. A native build has zero JS.

### Minimal impact on pledgepack binary

| Concern | Impact |
|---------|--------|
| Bundle size (output) | **Zero** — each target gets only its own code |
| Binary size (pledgepack itself) | **~200-500KB per adapter** — negligible |
| Performance | **Same or better** — Zig hot paths shared, adapter code only runs when invoked |
| Memory (runtime) | **Zero overhead** — unused adapters aren't loaded |
| Build time (pledgepack dev) | **Slightly longer** if compiling all adapters, but feature flags let you compile only what you need |

### npm distribution

Platform-specific packages (already implemented):
```
@pledgejs/pledgepack-darwin-arm64    # macOS Apple Silicon
@pledgejs/pledgepack-darwin-x64      # macOS Intel
@pledgejs/pledgepack-linux-x64       # Linux
@pledgejs/pledgepack-win32-x64       # Windows
```

Each platform package is ~5-15MB. Adding adapters adds ~200-500KB. Users don't notice.

Optional: adapter-specific packages:
```
@pledgejs/pledgepack-web             # web only, ~8MB
@pledgejs/pledgepack-rn              # RN only, ~9MB
@pledgejs/pledgepack-rye             # rye only, ~8MB
@pledgejs/pledgepack-all             # everything, ~12MB
```

npm's `optionalDependencies` auto-installs the right one. Users only download what they need.

---

## Roadmap — All Steps Complete

### ✅ Step 1: `adapter-rye`
- Wasm build orchestration (`wasm-pack`)
- HTML shell with hydration markers
- HMR (full re-instantiate + template-patch)
- **Result**: `pledge rye dev` for rye web apps ✅

### ✅ Step 2: `adapter-react-native`
- Platform-aware resolver (`.ios.js`, `.android.js`)
- Metro-compatible dev server protocol
- Hermes bytecode output (via Hermes CLI subprocess)
- HMR via WebSocket
- **Result**: `pledge rn dev` replaces Metro ✅

### ✅ Step 3: `adapter-rye-native`
- Cross-compile Rust to iOS/Android
- wgpu + winit on mobile
- Pledgepack orchestrates `cargo build --target aarch64-apple-ios`
- **Result**: `pledge rye build --target ios` → native app, no RN, no JS ✅

---

## Lessons Learned

- **Scope creep**: Each adapter was real work. Shipped `adapter-rye` first (smallest scope), then `adapter-react-native` (biggest adoption), then native targets — sequenced correctly.
- **Metro protocol**: Undocumented in places. Reading Metro's source code was necessary to match it exactly.
- **Hermes**: Used as subprocess rather than linking the C++ dependency into the Rust+Zig project.
- **iOS/Android tooling**: Cross-compiling Rust to mobile requires Xcode/Android SDK. Pledgepack detects and orchestrates these.
- **Native module bridging**: RN's bridge registration code generation requires knowing which native modules exist — handled via adapter configuration.

---

## Competitive landscape

No existing bundler handles all four targets:

| Tool | Web | RN | Native | Rust/Wasm | Language |
|------|-----|-----|--------|-----------|----------|
| Metro | No | Yes | No | No | JS |
| Vite | Yes | No | No | No | JS + esbuild |
| Webpack | Yes | Partial | No | No | JS |
| Turbopack | Yes | No | No | No | Rust |
| esbuild | Yes | No | No | No | Go |
| Parcel | Yes | No | No | No | JS + Rust |
| Trunk | No | No | No | Yes | Rust |
| **Pledgepack** | **Yes** | **Implemented** | **Implemented** | **Implemented** | **Rust + Zig** |

Pledgepack is the first universal bundler. Category-defining feature.
