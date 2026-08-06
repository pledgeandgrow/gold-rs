# Dual-Renderer Architecture

## Overview

rye supports **two rendering backends** from a single codebase:

| Backend | Renderer | Target | Use Case |
|---------|----------|--------|----------|
| **WebView** (default) | `DomRenderer` | WASM → browser/WebView | Web, mobile (Capacitor), desktop WebView |
| **Native** (opt-in) | `NativeRenderer` | wgpu + taffy + cosmic-text | Desktop exe, mobile native |

Both implement the same `Renderer` trait. Your app code (`template!`, signals, `rye-ui` components) is **identical** regardless of backend.

## How It Works

```
                    ┌─────────────┐
                    │  Your App   │  ← one codebase
                    │ (template!) │
                    │  (signals)  │
                    │  (rye-ui)   │
                    └──────┬──────┘
                           │ Element tree
                    ┌──────▼──────┐
                    │  Renderer   │  ← trait abstraction
                    │   trait     │
                    └──┬───────┬──┘
              ┌────────┘       └────────┐
     ┌────────▼────────┐    ┌──────────▼──────────┐
     │   DomRenderer   │    │   NativeRenderer    │
     │   (rye-html)    │    │   (rye-desktop)     │
     │   WASM + DOM    │    │   wgpu + taffy      │
     └─────────────────┘    └─────────────────────┘
```

## Feature Flags

### For library users (default — WebView)

```toml
[dependencies]
rye = { version = "0.1" }  # webview is default
```

### For native desktop apps

```toml
[dependencies]
rye = { version = "0.1", features = ["native"] }
```

### For the demo crate

```toml
[features]
default = ["webview"]
webview = []
native = ["dep:rye-desktop"]
```

## Entry Points

### WebView (WASM)

```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start() {
    use rye_core::mount;
    use rye_html::DomRenderer;

    let renderer = DomRenderer::new();
    renderer.setup_delegation();
    let scope = mount(|| build_app(), renderer);
    std::mem::forget(scope);
}
```

### Native (Desktop)

```rust
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub fn run_desktop() -> Result<(), Box<dyn std::error::Error>> {
    use rye_desktop::NativeRenderer;
    use rye_desktop::window::{run, WindowConfig};

    let config = WindowConfig {
        title: "My App".to_string(),
        width: 1024.0,
        height: 720.0,
        resizable: true,
    };

    run(config, |renderer| { /* render */ }, |input| { /* handle */ })
}
```

## Platform Abstraction

The `Platform` trait in `rye-core` abstracts over system APIs:

```rust
use rye_core::Platform;

fn save_file(platform: &dyn Platform, name: &str, contents: &str) {
    platform.write_file(name, contents);
}
```

Each backend provides its own implementation:
- **Web**: `WebPlatform` — uses browser APIs (fetch, localStorage, Notifications)
- **Desktop**: `NativePlatform` — uses OS APIs (std::fs, reqwest, notify-rust)
- **Mobile**: `MobilePlatform` — uses platform bridges (JNI, objc)

## Building

### Web (WASM)

```sh
wasm-pack build --target web --out-dir www/pkg --no-opt
```

### Native (Desktop exe)

```sh
cargo build --release --features native
```

### Mobile (Capacitor — WebView)

```sh
wasm-pack build --target web --out-dir www/pkg --no-opt
cd mobile && npx cap sync
npx cap run android  # or ios
```

## What's Shared (90%)

- All `template!` views
- All signal/state logic
- All `rye-ui` components
- All business logic
- All routing

## What's Platform-Specific (10%)

- Entry point (`start()` vs `run_desktop()`)
- Platform API implementations (filesystem, notifications, networking)
- Styling edge cases (some CSS features need manual implementation natively)
- Event mapping (DOM events vs winit events — handled inside renderer)
