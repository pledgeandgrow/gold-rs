# rye CI/CD Pipeline

## Workflows

| Workflow | Trigger | Purpose |
|---|---|---|
| `ci.yml` | Push to main, PRs | Format, clippy, tests (Linux/macOS/Windows), WASM build, security audit, docs, MSRV |
| `size-check.yml` | PRs touching crates | WASM bundle size measurement + report |
| `benchmarks.yml` | Push to main, PRs touching core/signals | Performance benchmarks with regression alerts |
| `release.yml` | Tag push (`v*.*.*`) | Pre-release checks → publish to crates.io → GitHub Release |

## CI Matrix

| Target | OS | What runs |
|---|---|---|
| Linux | ubuntu-latest | fmt, clippy, test, audit, docs, MSRV, WASM build, benchmarks |
| macOS | macos-latest | test |
| Windows | windows-latest | test |
| WASM | ubuntu-latest | cargo build --target wasm32-unknown-unknown, wasm-opt, size check |

## Secrets Required

| Secret | Used by | Purpose |
|---|---|---|
| `CARGO_REGISTRY_TOKEN` | release.yml | Publish to crates.io |
| `GITHUB_TOKEN` | benchmarks.yml, release.yml | Auto-generated, no setup needed |

## Branch Protection (Recommended)

Enable these rules on the `main` branch:

- [x] Require status checks to pass before merging:
  - Format Check
  - Clippy
  - Test (Linux)
  - Test (macOS)
  - Test (Windows)
  - WASM Build
  - Security Audit
  - Docs Build
  - MSRV Check
- [x] Require branches to be up to date before merging
- [x] Require conversation resolution before merging
- [x] Require linear history
- [x] Do not allow bypassing the above settings

## Dependabot

Dependabot is configured to check for dependency updates weekly.
See `.github/dependabot.yml`.
