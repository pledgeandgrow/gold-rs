# Contributing to rye

Thank you for your interest in contributing to rye! This document covers the basics.

## Getting Started

1. Find an issue labeled `good first issue` or `help wanted`
2. Comment on the issue to claim it
3. Fork the repo and create a feature branch: `git checkout -b feature/my-feature`
4. Make your changes
5. Run `cargo fmt`, `cargo clippy`, `cargo test`
6. Open a PR with a clear description

## Code Style

- Follow `rustfmt` defaults (enforced in CI)
- Follow `clippy` with no warnings (enforced in CI)
- Public API must have rustdoc comments
- No `unwrap()` or `expect()` in production code
- No `unsafe` without a SAFETY comment

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(router): add nested route support
fix(signals): batch updates correctly in async context
docs(reactivity): add dependency tracking explanation
```

## PR Checklist

- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` passes
- [ ] New tests added for new functionality
- [ ] Documentation updated
- [ ] CI passes on all targets

## RFC Process

For significant changes (new API, breaking changes, architecture decisions), submit an RFC. See [Governance](docs/07-GOVERNANCE.md) for the full process.

## Code of Conduct

See [Governance](docs/07-GOVERNANCE.md) for our Code of Conduct.
