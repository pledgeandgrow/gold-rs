# rye-demo mobile

Wraps the rye WASM demo in a native WebView for testing on Android and iOS.

## Prerequisites

### Android
- **Android Studio** (includes SDK + emulator)
  - Download: https://developer.android.com/studio
- After install: Tools → SDK Manager → install an SDK (e.g. API 34)
- Create an emulator: Tools → Device Manager → Create Device → Pixel 7

### iOS (macOS only)
- **Xcode** (from App Store)
- **CocoaPods**: `sudo gem install cocoapods`
- iOS Simulator is bundled with Xcode

## Quick start

```sh
# 1. Build WASM and sync into native projects
npm run sync

# 2. Open in Android Studio
npm run open:android
#   → Select an emulator/device, click Run ▶

# 3. Open in Xcode (macOS only)
npm run open:ios
#   → Select a simulator, click Run ▶
```

## Workflow

After changing Rust code:

```sh
npm run sync          # rebuilds WASM + copies to both platforms
npm run open:android  # reopen and run
```

Or target a single platform:

```sh
npm run sync:android  # rebuild + sync Android only
npm run sync:ios      # rebuild + sync iOS only
```

## Running on a physical device

### Android
1. Enable USB debugging on your phone (Settings → Developer Options)
2. Connect via USB
3. `npm run open:android` → select your device → Run

### iOS
1. Connect iPhone via USB
2. `npm run open:ios` → select your device → Run
3. First run: trust the developer cert in Settings → General → VPN & Device Management

## Project structure

```
mobile/
├── capacitor.config.json   # Capacitor config (appId, webDir)
├── package.json            # npm scripts for build/sync/open
├── android/                # Android Studio project (auto-generated)
└── ios/                    # Xcode project (auto-generated)

../www/                     # WASM output (webDir target)
├── index.html
└── pkg/
    ├── rye_demo.js
    └── rye_demo_bg.wasm
```

## How it works

1. `wasm-pack build` compiles `rye-demo` to WASM → `www/pkg/`
2. `cap sync` copies `www/` into `android/app/src/main/assets/public/` and `ios/App/App/public/`
3. The native app opens a WebView that loads `index.html`
4. `index.html` imports the WASM module and calls `start()`
5. rye's `DomRenderer` renders directly into the WebView DOM

The entire rye app runs client-side in the WebView — no server needed.
