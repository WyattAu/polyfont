# Contributing to Polyfont

Thank you for your interest in contributing. This guide covers the essentials for working on the Polyfont codebase.

## Development Setup

**Prerequisites:**

- Rust 1.85 or later (`rustup update`)
- Cargo (bundled with Rust)

**Building:**

```
git clone https://github.com/WyattAu/polyfont.git
cd polyfont
cargo build
```

**Pre-commit hooks:** Install [pre-commit](https://pre-commit.com/) and run `pre-commit install` to enable formatting and linting checks on every commit. Hook definitions are in `.pre-commit-config.yaml`.

## Code Style

- Run `cargo fmt --all` before committing. CI enforces formatting.
- Run `cargo clippy --workspace -- -D warnings` and fix all warnings. CI treats clippy warnings as errors.
- Do not use `.unwrap()` or `.expect()` in library crate code. Use `anyhow::Result` or `thiserror` for error propagation. Binaries (`polyfont-cli`, `polyfont-lsp`) may use `.expect()` at the top level with descriptive messages.

## Testing

```
cargo test --workspace
```

All 96 tests must pass. CI runs the full test suite on every push and PR.

**Property-based testing** with `proptest` is encouraged for parsing, scope matching, and configuration logic. Add `proptest` as a dev-dependency in the relevant crate's `Cargo.toml`.

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]
```

Types: `feat`, `fix`, `docs`, `perf`, `refactor`, `test`, `chore`, `ci`

Examples:

```
feat(parse): add support for TOML array of tables in rules
fix(lsp): handle empty scope string without panic
docs: update CONTRIBUTING with stability policy
```

## Pull Request Process

1. All CI checks must pass (fmt, clippy, tests).
2. The PR description must summarize the changes and their motivation.
3. Link related issues with `Fixes #<n>` or `Closes #<n>`.
4. Keep PRs focused. Split unrelated changes into separate PRs.

## Stability Policy

Polyfont makes the following stability guarantees:

### Config format (`.polyfont.toml`)

Stable. No breaking changes to the configuration schema without a major version bump. New fields and rule properties may be added as optional entries with sensible defaults.

### CLI interface

Stable. Existing subcommands and flags will not be removed without a deprecation period. New subcommands may be added in minor releases.

### LSP protocol

Stable. Custom notifications and requests will not change semantics without LSP initialization negotiation. Adhere to the [LSP specification](https://microsoft.github.io/language-server-protocol/).

### Crate public APIs

Follow Rust [SemVer](https://semver.org/). Public types, traits, and functions will not be removed or have their signatures changed without a major version bump. CI enforces this with `cargo-semver-checks`.

### Editor extension APIs

Follow the versioning conventions of each target platform (VS Code Marketplace, Helix runtime, Neovim plugin ecosystem, Sublime Package Control, Zed extension manager).

## Release Process

Releases are tag-based. To publish:

1. Bump version in `Cargo.toml` workspace `[workspace.package]`.
2. Tag the commit as `vX.Y.Z` and push.
3. CI builds platform binaries, publishes crates to crates.io, and updates editor extension marketplaces.

## Project Structure

```
polyfont/
  crates/
    polyfont-core/     Core types and traits for font resolution
    polyfont-config/    .polyfont.toml parsing and validation
    polyfont-scope/     TextMate scope matching and resolution
    polyfont-parse/     Syntax tree construction and tokenization
    polyfont-fonts/     Font discovery, loading, and metadata
    polyfont-themes/    Theme definitions and color palette handling
    polyfont-cli/       Command-line interface binary
    polyfont-lsp/       Language Server Protocol server
  editors/
    vscode/             VS Code extension
    neovim/              Neovim plugin
    helix/               Helix integration
    sublime/             Sublime Text package
    zed/                 Zed extension
```

Licensed under Apache-2.0. By contributing, you agree that your contributions will be licensed under the same terms.
