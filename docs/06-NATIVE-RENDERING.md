# Native Rendering Paths — GPU Rendering Strategy

> Goal 7: Research and document options for truly native desktop/mobile rendering (no WebView).

---

## Decision: `wgpu` + `winit` + `taffy` for Native GPU Rendering

We adopt a **custom GPU renderer** stack:

| Layer | Crate | Purpose |
|---|---|---|
| Windowing | `winit` (or `tao`) | Window creation, event loop, multi-window |
| GPU rendering | `wgpu` | Cross-platform GPU abstraction (Vulkan/Metal/DX12/WebGPU) |
| Text rendering | `cosmic-text` | Text shaping, layout, glyph rasterization |
| Layout engine | `taffy` | Flexbox + Grid layout (same semantics as CSS) |
| 2D shapes | `lyon` | Path tessellation for borders, backgrounds, shadows |

---

## Why Not WebView?

| Criterion | WebView | Native GPU |
|---|---|---|
| Performance | Medium (DOM overhead, JS bridge) | High (direct GPU) |
| Memory | High (full browser engine) | Low (only what's needed) |
| Bundle size | Large (bundles browser engine or relies on system) | Small (just GPU code) |
| Consistency | Varies (WebKit, Chromium, EdgeHTML differ) | Identical on all platforms |
| Startup time | Slow (browser init) | Fast (~50ms) |
| Custom rendering | Limited (CSS only) | Unlimited (shaders, custom paint) |
| Dependencies | System WebView (version fragmentation) | None (wgpu handles backend) |

**Conclusion:** WebView is a fallback, not the primary desktop/mobile renderer.

---

## Architecture

```
┌─────────────────────────────────────────────┐
│              Component Tree                  │
│         (rye-core, platform-agnostic)        │
├─────────────────────────────────────────────┤
│           Renderer Trait                     │
├──────────────────┬──────────────────────────┤
│  DomRenderer     │  NativeRenderer           │
│  (web/WASM)      │  (desktop/mobile)         │
│  web-sys DOM     │  wgpu + winit + taffy     │
├──────────────────┼──────────────────────────┤
│  Browser DOM     │  GPU (Vulkan/Metal/DX12)  │
└──────────────────┴──────────────────────────┘
```

### NativeRenderer Internals

```
NativeRenderer
├── WindowManager (winit)
│   ├── Window creation / multi-window
│   ├── Event loop (mouse, keyboard, touch, resize)
│   └── DPI / scaling
├── LayoutEngine (taffy)
│   ├── Flexbox layout
│   ├── Grid layout
│   ├── Absolute positioning
│   └── Text layout (via cosmic-text)
├── RenderPipeline (wgpu)
│   ├── Background rendering (rounded rects, gradients)
│   ├── Border rendering (solid, dashed, rounded)
│   ├── Shadow rendering (box-shadow)
│   ├── Text rendering (glyph atlas + cosmic-text)
│   ├── Image rendering (texture sampling)
│   └── Clip / overflow / scroll
├── InputManager
│   ├── Hit testing (which element is under cursor)
│   ├── Event delegation (dispatch to correct element)
│   ├── Focus management
│   └── Touch / gesture recognition
└── AccessibilityBridge
    ├── Windows: UIAutomation
    ├── macOS: NSAccessibility
    └── Linux: AT-SPI
```

---

## Layout Engine — `taffy`

`taffy` provides flexbox and grid layout in pure Rust. It maps CSS layout concepts to native rendering:

```rust
// rye maps template! layout properties to taffy styles
template! {
    div {
        style: {
            display: "flex",
            flex_direction: "column",
            gap: "16px",
            padding: "24px",
            align_items: "center",
        },
        div { style: { flex: "1" }, "Content" }
    }
}
```

### Supported layout properties

| Property | taffy support | Priority |
|---|---|---|
| `display: flex` | Yes | P0 |
| `display: grid` | Yes | P0 |
| `flex_direction` | Yes | P0 |
| `align_items` / `justify_content` | Yes | P0 |
| `flex_grow` / `flex_shrink` / `flex_basis` | Yes | P0 |
| `gap` | Yes | P0 |
| `padding` / `margin` | Yes | P0 |
| `width` / `height` | Yes | P0 |
| `min_width` / `max_width` | Yes | P1 |
| `position: absolute` | Yes | P1 |
| `position: relative` | Yes | P1 |
| `overflow: hidden` | Yes (clip) | P1 |
| `overflow: scroll` | Custom (scroll container) | P2 |

---

## Text Rendering — `cosmic-text`

`cosmic-text` (from System76's COSMIC desktop) provides:
- Font loading (system fonts + bundled fonts)
- Text shaping (HarfBuzz-backed)
- Text layout (wrapping, alignment, bidi)
- Glyph rasterization (to texture atlas)
- Cursor / selection support

### Why cosmic-text?

| Alternative | Pros | Cons |
|---|---|---|
| `cosmic-text` | Full-featured, active development, system fonts | Relatively new |
| `swash` | Lightweight, fast | Less feature-complete |
| `rusttype` | Mature | No shaping, basic layout |
| `ab_glyph` | Simple | No shaping, no complex text |

**Decision:** Use `cosmic-text` as primary, with `swash` as fallback for platforms where `cosmic-text` isn't available.

---

## GPU Rendering Pipeline — `wgpu`

### Render passes

1. **Background pass** — Fill rounded rectangles for element backgrounds (solid, gradient, image)
2. **Border pass** — Stroke rounded rectangles for borders
3. **Shadow pass** — Render blurred shadows (gaussian blur via compute shader or two-pass)
4. **Text pass** — Render glyphs from texture atlas
5. **Image pass** — Render image textures
6. **Clip pass** — Apply clipping regions (overflow: hidden, border-radius)

### Shader approach

Use WGSL (WebGPU Shading Language) for all shaders:
- `bg.wgsl` — Rounded rectangle fill with gradient support
- `border.wgsl` — Rounded rectangle stroke
- `shadow.wgsl` — Gaussian blur shadow
- `text.wgsl` — Glyph atlas sampling with SDF (signed distance field) for crisp text at any scale
- `image.wgsl` — Texture sampling with filtering

### Glyph atlas

- Pre-rasterize glyphs at common sizes (12, 14, 16, 18, 24, 32, 48px)
- Cache in a texture atlas (1024x1024 or 2048x2048)
- Dynamic atlas growth — add new glyphs on demand
- SDF (signed distance field) encoding for resolution-independent text

---

## Mobile Considerations

### iOS

| Aspect | Approach |
|---|---|
| Window | `winit` creates a `UIView` |
| GPU | Metal via `wgpu` |
| Text | `cosmic-text` (system fonts via Core Text fallback) |
| Touch | `winit` touch events → unified `TouchEvent` |
| Lifecycle | `UIApplicationDelegate` bridge — foreground/background/suspend |
| Safe area | `safeAreaInsets` → layout padding |
| Keyboard | Virtual keyboard overlay → resize event |

### Android

| Aspect | Approach |
|---|---|
| Window | `winit` creates a `SurfaceView` |
| GPU | Vulkan via `wgpu` |
| Text | `cosmic-text` (system fonts via NDK) |
| Touch | `winit` touch events → unified `TouchEvent` |
| Lifecycle | `Activity` lifecycle bridge — onPause/onResume/onDestroy |
| Safe area | Window insets → layout padding |
| Keyboard | Soft keyboard → resize event |
| APK packaging | `rye build --target android` → APK via `cargo-apk` or custom |

---

## Performance Targets (Native)

| Metric | Target |
|---|---|
| Cold start to first frame | <100ms |
| Frame time (60fps budget) | <16ms |
| Frame time (120fps budget) | <8ms |
| Text layout (1000 chars) | <1ms |
| Layout (1000 nodes) | <5ms |
| Memory per element | <500 bytes |
| GPU memory (glyph atlas) | <4MB |

---

## Fallback: WebView Renderer

For platforms where `wgpu` isn't available (rare), or for hybrid apps:

```rust
// In Cargo.toml, choose renderer at build time
#[cfg(feature = "native-gpu")]
use rye::NativeRenderer;

#[cfg(feature = "webview")]
use rye::WebViewRenderer;
```

`WebViewRenderer` uses the same `Renderer` trait, so the component tree is identical. Only the rendering backend changes.

---

## Comparison with Competitors

| Feature | Dioxus | Tauri | Electron | Flutter | `rye` |
|---|---|---|---|---|---|
| Desktop rendering | WebView | WebView | WebView | Native GPU (Skia) | Native GPU (wgpu) |
| Mobile rendering | WebView (experimental) | N/A | N/A | Native GPU (Skia) | Native GPU (wgpu) |
| Memory usage | Medium | Medium | High | Low | Low |
| Startup time | Medium | Medium | Slow | Fast | Fast |
| Bundle size | Small | Small | Large | Medium | Small |
| Layout engine | CSS (browser) | CSS (browser) | CSS (browser) | Custom (Skia) | taffy (flexbox/grid) |
| Text rendering | Browser | Browser | Browser | Skia + libtxt | cosmic-text |
| Consistency | Varies (WebView) | Varies | Varies | Identical | Identical |

---

## Implementation Priority

| Priority | Component | Effort | Dependency |
|---|---|---|---|
| P0 | `winit` window + event loop | Medium | None |
| P0 | `taffy` layout integration | Medium | None |
| P0 | `wgpu` basic rect rendering | High | winit |
| P0 | `cosmic-text` text rendering | High | wgpu |
| P1 | Border + shadow rendering | Medium | wgpu basic |
| P1 | Image/texture rendering | Medium | wgpu basic |
| P1 | Scroll containers | Medium | taffy |
| P1 | Hit testing + event dispatch | Medium | taffy |
| P2 | Accessibility bridge | High | winit |
| P2 | Mobile lifecycle integration | Medium | winit |
| P2 | WebView fallback renderer | Medium | web-sys |
| P3 | Custom shaders / effects | Low | wgpu |
| P3 | Multi-window support | Medium | winit |

---

*This document defines the native rendering architecture. **Implemented** in `rye-desktop` (wgpu + winit + taffy + cosmic-text), `rye-mobile` (iOS/Android targets), and `rye-core/src/rendering/` (WebGPU, virtual scroll, observers, media queries, view transitions, container queries, web animations, DPI, multi-window).*
