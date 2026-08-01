# Project Governance

> Goal 8: Establish project governance, RFC process, contribution guidelines, and code of conduct.

---

## Governance Model

`rye` uses a **meritocratic governance model** with transparent decision-making, inspired by Rust's governance structure.

### Roles

| Role | Description | How to get it |
|---|---|---|
| **Contributor** | Anyone who submits a PR, issue, or discussion | Submit a contribution |
| **Member** | Regular contributors with write access to branches | Consistent quality contributions over time, nominated by a Maintainer |
| **Maintainer** | Core team members with merge access to main | Nominated by existing Maintainers, consensus approval |
| **Lead** | Final decision authority, release management | Project founders initially; elected by Maintainers |

### Decision Making

| Decision Type | Who Decides | Process |
|---|---|---|
| Bug fixes, minor features | Maintainer | PR review + merge |
| Major features, API changes | Maintainers | RFC → discussion → merge |
| Breaking changes | Maintainers + Lead | RFC → consensus → version bump |
| Governance changes | All Maintainers | RFC → supermajority vote |
| Release decisions | Lead | Lead decides timing, Maintainers sign off |

---

## RFC Process

All significant changes go through the RFC (Request for Comments) process.

### When to submit an RFC

- New public API or changes to existing API
- New crate or major module
- Changes to the reactivity model, rendering strategy, or template syntax
- Breaking changes (even in pre-1.0)
- Governance changes

### When NOT to submit an RFC

- Bug fixes (just open a PR)
- Internal refactors with no API change
- Documentation improvements
- Test additions

### RFC Lifecycle

```
Draft → Submitted → Under Review → Accepted/Rejected → Implemented → Merged
```

1. **Draft** — Author writes the RFC using the template
2. **Submitted** — PR opened to `rfcs/` directory
3. **Under Review** — Community discusses in PR comments and GitHub Discussions
4. **Accepted/Rejected** — Maintainers decide (typically 2-week review period)
5. **Implemented** — Author or volunteer implements in a feature branch
6. **Merged** — Implementation merged to main, RFC moved to `rfcs/accepted/`

### RFC Template

```markdown
# RFC: [Title]

## Start Date
[YYYY-MM-DD]

## Summary
[One paragraph summary]

## Motivation
[Why is this needed? What problem does it solve?]

## Detailed Design
[Technical details, API surface, examples, trade-offs]

## Alternatives Considered
[What other approaches were considered and why they were rejected]

## Unresolved Questions
[Open questions to resolve during implementation]

## Backwards Compatibility
[Impact on existing code, migration path if any]
```

---

## Code of Conduct

### Our Pledge

We are committed to providing a friendly, safe, and welcoming environment for all, regardless of level of experience, gender identity and expression, sexual orientation, disability, personal appearance, body size, race, ethnicity, age, religion, nationality, or other similar characteristics.

### Our Standards

**Positive behavior:**
- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Unacceptable behavior:**
- Trolling, insulting/derogatory comments, and personal or political attacks
- Public or private harassment
- Publishing others' private information without explicit permission
- Other conduct which could reasonably be considered inappropriate

### Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported to the Conduct Team (`conduct@rye.rs`). All complaints will be reviewed and investigated promptly and fairly.

---

## Contributing Guidelines

### Getting Started

1. Find an issue labeled `good first issue` or `help wanted`
2. Comment on the issue to claim it
3. Fork the repo and create a feature branch: `git checkout -b feature/my-feature`
4. Make your changes following the code style below
5. Run `cargo fmt`, `cargo clippy`, `cargo test`
6. Open a PR with a clear description

### Code Style

- Follow `rustfmt` defaults (enforced in CI)
- Follow `clippy` with no warnings (enforced in CI)
- Use descriptive names — no single-letter variables except in tight loops
- Public API must have rustdoc comments
- No `unwrap()` or `expect()` in production code — use proper error handling
- No `unsafe` without a SAFETY comment explaining why it's sound

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(router): add nested route support
fix(signals): batch updates correctly in async context
docs(reactivity): add dependency tracking explanation
refactor(core): simplify renderer trait
test(forms): add validation edge case tests
chore(ci): add wasm size check
```

### PR Checklist

- [ ] Code follows style guidelines (`cargo fmt`, `cargo clippy`)
- [ ] Tests pass (`cargo test`)
- [ ] New tests added for new functionality
- [ ] Documentation updated (rustdoc + guides if applicable)
- [ ] No breaking changes, OR breaking changes documented with migration guide
- [ ] CI passes on all targets (wasm, linux, macos, windows)

### Review Process

1. Automated CI must pass (build, test, fmt, clippy, size check)
2. At least one Maintainer review required
3. For major changes, two Maintainer reviews required
4. Reviewer should check: correctness, performance, API design, test coverage, docs
5. Author addresses feedback by pushing to the same branch

---

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **0.x.x** — Pre-release. Breaking changes allowed in minor versions.
- **1.0.0+** — Stable. Breaking changes only in major versions.
- Each crate can version independently, but should stay in sync for major releases.

### Release Checklist

1. All planned issues for the release are closed
2. All tests pass on all platforms
3. `CHANGELOG.md` is updated
4. Benchmarks show no regressions
5. WASM size is within budget
6. Create release branch: `release/v0.x.x`
7. Tag: `git tag v0.x.x`
8. Publish to crates.io: `cargo publish` (per crate, in dependency order)
9. Publish GitHub Release with changelog and migration guide
10. Update documentation site with new version

### LTS Policy (Post-1.0)

- Each major version (1.x, 2.x) gets **2 years** of support
- Security fixes backported to supported major versions
- Non-security bug fixes only applied to latest major version
- End-of-life announced 6 months in advance

---

## Community Infrastructure

| Platform | Purpose | URL |
|---|---|---|
| GitHub | Code, issues, PRs, RFCs | `github.com/rye-rs/rye` |
| GitHub Discussions | Q&A, announcements, ideas | (on GitHub repo) |
| Discord | Real-time chat, help, showcase | `discord.gg/rye-rs` |
| Documentation Site | Guides, API docs, examples | `rye.rs` |
| crates.io | Package publishing | `crates.io/crates/rye` |

### Communication Guidelines

- **Issues** — Bug reports and feature requests (use templates)
- **Discussions** — Questions, ideas, show-and-tell
- **Discord** — Quick help, community bonding, dev coordination
- **RFCs** — Formal proposals for significant changes
- **Blog** — Release announcements, technical deep dives, tutorials

---

*This document establishes the governance framework. The project has completed all 150 implementation goals across 14 phases. Governance processes (RFCs, releases, contributing guidelines) are active.*
