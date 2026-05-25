# Polyfont Roadmap v2

Per-token font highlighting for code editors. This document tracks the path from
the current v0.10.0 release through v1.0 production and beyond.

**Current version:** 0.10.0 (2026-05-25)
**Repository:** https://github.com/WyattAu/polyfont
**License:** Apache-2.0

---

## Completed Work (v0.1 -- v0.10)

All milestones through v0.10 are shipped. The following summary captures what each
release delivered and the cumulative state of the project.

### Release History

| Version | Focus Area | Key Addition | Tests |
|---------|-----------|--------------|-------|
| v0.1.0 | Core foundation | 5 crates, VSCode + Neovim extensions, CI/CD | 16 |
| v0.2.0 | Parsing | polyfont-parse, tree-sitter (10 languages) | 30 |
| v0.3.0 | Neovim metadata | Font metadata API, `:PolyfontMetadata` | -- |
| v0.4.0 | Font management | polyfont-fonts, cross-platform discovery | 46 |
| v0.5.0 | Performance | TrieScopeResolver O(k) lookup, benchmarks | 53 |
| v0.6.0 | Themes | polyfont-themes, import/export/registry | 74 |
| v0.7.0 | Editor docs | Zed, Helix, Sublime integration guides | -- |
| v0.8.0 | Documentation | Architecture, migration, SemVer CI, release workflow | -- |
| v0.9.0 | Feature expansion | Variable font axes, rendering skeleton, JetBrains, accessibility | 117 |
| v0.10.0 | Font/theme CLI | `font install/preview`, `theme import/export`, remote registry | 120 |

### Current Project State

**Crate inventory (9 crates):**

| Crate | Role | Status |
|-------|------|--------|
| polyfont-core | TokenInfo, FontSpec, FontRule, ScopeMatchEngine, benchmarks | Stable |
| polyfont-config | TOML config loading, PolyfontConfig, ConfigLoader | Stable |
| polyfont-scope | ScopePattern, ScopeSelector, TrieScopeResolver, ScopeTree | Stable |
| polyfont-parse | Tree-sitter token parser, 10 language grammars, naive fallback | Stable |
| polyfont-fonts | Cross-platform font discovery, download, caching | Stable |
| polyfont-themes | Theme import/export, 5 built-in themes, accessibility checker | Stable |
| polyfont-lsp | tower-lsp server, tree-sitter-first tokenizer, naive fallback | Stable |
| polyfont-cli | check, vscode, neovim, kitty, dump, font, theme subcommands | Stable |
| polyfont-render | Rendering engine skeleton (GPU + software paths) | Experimental |

**Editor integrations (6):**

| Editor | Integration Type | Maturity |
|--------|-----------------|----------|
| VSCode | Native extension (TypeScript, textMateRules fontFamily API) | Stable |
| Neovim | Lua plugin + LSP client, font metadata for GUIs | Stable |
| Zed | WASM extension (text_style_overrides) | Skeleton |
| Helix | Kitty symbol_map approximation | Experimental |
| Sublime Text | Python plugin (.sublime-theme generation) | Skeleton |
| JetBrains | Kotlin IntelliJ plugin | Skeleton |

**Infrastructure:**
- 120 tests passing across all crates
- Cross-platform CI (Linux/macOS/Windows, 5 binary targets)
- GitHub Pages documentation site
- SemVer checking via cargo-semver-checks
- Pre-commit hooks (fmt, clippy, test)
- publish.sh for crates.io dry-run and release

**Technical capabilities delivered:**
- Trie-based O(k) scope matching with specificity-aware resolution
- Font download from 6 registries (GitHub, Google Fonts, URL)
- Theme import from TextMate .tmTheme and VSCode JSON
- Remote theme registry (3 built-in themes)
- Variable font axis control (NamedAxis, AxisValue, CSS variation settings)
- Accessibility checker with dyslexia-friendly recommendations
- LSP font suggestions (polyfont/suggestFonts)
- Collaborative theme infrastructure (ThemeShare, ThemeLockfile)
- Rendering engine skeleton with GPU (wgpu + glyphon) and software (softbuffer + tiny-skia) paths

---

## Path to v1.0 Production

The following milestones bridge the gap between the current feature-complete v0.10
and a production-ready v1.0 release. Each milestone has clear deliverables, a
technical approach, and measurable success criteria.

### Milestone Dependency Graph

```
v0.11 (render) ──────────────────────────────────────┐
v0.12 (LSP hardening) ───────────────────────────────┤
v0.13 (Neovim native) ───────────────────────────────┤
v0.14 (performance at scale) ── depends on v0.12 ────┤
v0.15 (editor testing) ──────── depends on v0.12 ────┤
v0.16 (crates.io publication) ─ depends on v0.14 ────┤
v0.17 (security audit) ──────── depends on v0.16 ────┤
v1.0 (production release) ───── depends on v0.17 ────┘
```

---

### v0.11 -- Render Engine Completion

**Goal:** Complete the polyfont-render crate so that editors and terminals without
native per-token font support can render multi-font output via the polyfont
rendering pipeline.

**Key Deliverables:**

- GPU rendering path: wgpu-based compositor that maintains a per-font glyph atlas
  and renders annotated text with per-token font switching
- Software rendering path: CPU fallback using tiny-skia + fontdb for environments
  without GPU access (headless CI, remote terminals)
- Font atlas management: automatic atlas packing, eviction of unused glyphs,
  support for at least 8 concurrent font families per frame
- Baseline alignment: consistent baseline across fonts with different metrics,
  configurable vertical centering strategy
- Ligature handling: detect and preserve ligatures within a single font family;
  split ligatures at font boundaries with configurable fallback
- CLI `polyfont render` subcommand: render a source file to PNG/SVG output
- 30+ new tests for the rendering pipeline

**Technical Approach:**

1. Extend `RenderEngine` in polyfont-render with a concrete `render_document()`
   method that accepts `Vec<(TokenInfo, FontAssignment)>` and produces a
   `RenderOutput` (pixel buffer or SVG path set).
2. Implement `GlyphCache` backed by a texture atlas (GPU path) or a HashMap of
   rasterized bitmaps (software path). Use `cosmic-text` for shaping and
   `glyphon`/`swash` for rasterization.
3. Handle baseline alignment by computing the union ascent/descent across all
   active fonts and aligning each glyph to the computed baseline.
4. For ligatures, query `cosmic-text` for shaping runs. If a ligature span
   crosses a font boundary, break the span and render each sub-span independently.
5. The `polyfont render` CLI subcommand reads a source file, runs the full
   tokenize-resolve pipeline, and writes the output to a file.

**Dependencies and Risks:**

- Dependencies: wgpu, glyphon, cosmic-text, tiny-skia, fontdb (already in
  Cargo.toml behind feature gates)
- Risk: GPU driver compatibility varies across platforms; the software path must
  be a complete fallback, not a stub
- Risk: Ligature splitting across font boundaries is an open typographic problem;
  accept imperfect results for v0.11 and document limitations

**Success Criteria:**

- `polyfont render test.rs -o output.png` produces a PNG with visually correct
  per-token font rendering for a Rust file with 3+ font families
- Software path produces identical output to GPU path (pixel-diff within 1% tolerance)
- Rendering a 10,000-line file completes in <500ms
- Memory usage stays under 200MB for files with 10+ concurrent font families

**Estimated Timeline:** 4-5 weeks

**Complexity:** 4/5

---

### v0.12 -- LSP Production Hardening

**Goal:** Harden the LSP server for daily-driver use with proper incremental
parsing, request cancellation, and robust tree-sitter integration.

**Key Deliverables:**

- Incremental re-parsing on `textDocument/didChange` using tree-sitter's edit API
  instead of full document re-parse
- Request cancellation via `$/cancelRequest` support; long-running tokenization
  jobs check a cancellation token between tokens
- Proper UTF-16 to UTF-8 offset conversion for LSP position fields (the LSP spec
  uses UTF-16 code units; tree-sitter uses byte offsets)
- Document synchronization: track document versions and discard stale results
- Error recovery: if tree-sitter parsing fails for a document, fall back to the
  naive tokenizer and log a warning (no silent failures)
- LSP compliance: pass the existing LSP conformance test suite for
  `textDocument/didOpen`, `textDocument/didChange`, and custom notifications
- 20+ new LSP-specific tests

**Technical Approach:**

1. Add a `ParserState` struct per open document containing the tree-sitter
   `Parser`, `Tree`, and the document version. Store in `ServerState.documents`
   as `HashMap<Url, DocumentState>` where `DocumentState` gains a
   `parser_state: Option<ParserState>` field.
2. On `didChange`, compute the tree-sitter `InputEdit` from the LSP content
   change ranges. Convert LSP UTF-16 positions to byte offsets using a
   `PositionConverter` utility that maintains a line-start offset table.
   Apply the edit to the existing tree, then re-highlight only the changed
   region.
3. Implement cancellation by passing a `CancellationToken` through the
   tokenization pipeline. Check the token every N tokens (tunable, default 1000).
   If cancelled, return a partial result and let the client re-request.
4. Add `$/cancelRequest` handler in the tower-lsp service that sets the
   cancellation flag for in-flight requests.

**Dependencies and Risks:**

- Dependencies: tree-sitter >= 0.24, tower-lsp (already in use)
- Risk: UTF-16/UTF-8 conversion bugs for non-ASCII text (emoji, CJK, combining
  marks); needs exhaustive test coverage with Unicode edge cases
- Risk: Incremental parsing correctness is critical; a single wrong edit offset
  produces a corrupt tree. Use tree-sitter's own edit application rather than
  manual tree manipulation

**Success Criteria:**

- Single-character edit in a 10,000-line file triggers re-tokenization of the
  affected region only, completing in <5ms
- `$/cancelRequest` cancels an in-flight tokenization within 1ms of receipt
- All Unicode edge cases (emoji, combining marks, RTL, CJK) produce correct
  token positions
- No panics or hangs under concurrent document modification (stress test with
  100 rapid edits)
- LSP server runs for 8 hours under typical usage without memory growth

**Estimated Timeline:** 3-4 weeks

**Complexity:** 3/5

---

### v0.13 -- Neovim Native Font Support

**Goal:** Achieve per-token font rendering in Neovim GUI frontends through
either an upstream neovim core contribution or a GUI bridge approach.

**Key Deliverables:**

- Neovim RFC filed at neovim/neovim proposing `font` field in `nvim_set_hl()`
  (draft RFC already exists at `docs/neovim-font-rfc.md`)
- Working patch for at least one Neovim GUI frontend (Goneovim or Neovide) that
  renders distinct fonts per scope
- Neovim plugin update to set the `font` highlight attribute when the GUI
  supports it, with silent degradation for terminals
- Compatibility test matrix across GUI frontends
- Documentation: supported GUIs, setup instructions, known limitations

**Technical Approach:**

1. Refine and submit the existing RFC (`docs/neovim-font-rfc.md`) to
   neovim/neovim. The RFC proposes adding an optional `font` string field to
   highlight group definitions. This is a non-breaking additive change; GUIs
   that don't read the field are unaffected.
2. If the RFC is accepted or shows signs of progress, implement the neovim core
   change and submit as a PR. If the RFC is deferred, maintain an out-of-tree
   patch set.
3. For immediate usability, build a GUI bridge: a thin wrapper around a
   supported GUI (Neovide preferred, since it's Rust-based and uses Skia for
   rendering) that reads the font metadata from `vim.g.polyfont_font_map` and
   applies per-highlight-group font switching during text layout.
4. Update `editors/neovim/lua/polyfont/highlights.lua` to conditionally set the
   `font` attribute when `vim.g.polyfont_gui_support` is detected.

**Dependencies and Risks:**

- Dependencies: Neovim RFC review (external, uncontrollable timeline), GUI
  maintainer cooperation
- Risk: RFC rejected or indefinitely deferred. Mitigation: the GUI bridge
  approach works without any neovim core changes
- Risk: Neovide font switching performance may be poor without proper caching.
  Mitigation: benchmark early and implement a font cache in the bridge layer
- Risk: Goneovim may have low maintainer activity. Verify before committing

**Success Criteria:**

- At least one Neovim GUI renders distinct fonts per syntax scope in a demo
- The Neovim plugin works unchanged in terminal mode (no regressions)
- RFC is submitted with clear API proposal and backward compatibility argument
- Documentation lists which GUIs are supported and how to enable the feature

**Estimated Timeline:** 5-8 weeks (includes upstream review wait time)

**Complexity:** 4/5

---

### v0.14 -- Performance at Scale

**Goal:** Establish comprehensive benchmarks, optimize for large codebases,
and validate that polyfont handles 500+ rule profiles without degradation.

**Depends on:** v0.12 (LSP hardening -- incremental parsing is a prerequisite
for performance measurement under realistic edit patterns)

**Key Deliverables:**

- Benchmark suite covering all crates:
  - Scope resolution throughput (tokens/sec) at 10/50/200/500/1000 rules
  - Full document tokenization pipeline (end-to-end) at 1K/10K/50K lines
  - Config loading and engine construction
  - LSP request handling latency
  - Memory allocation profiles (via dhat or jemalloc profiling)
- Optimization of hot paths identified by benchmarks:
  - Scope trie construction for large rule sets
  - Token allocation patterns (reduce Vec resizing)
  - Config parsing for files with 500+ rules
- Latency budget documentation: per-component latency targets and measured values
- Regression detection: CI fails if any benchmark regresses >10% from baseline
- 500-rule stress test profile: synthetic config with maximum realistic complexity

**Technical Approach:**

1. Extend `crates/polyfont-core/benches/` with parameterized benchmarks.
   Use criterion's `BenchmarkGroup` to test multiple input sizes in one group.
   Add `crates/polyfont-lsp/benches/` for end-to-end LSP latency measurement
   using a mock client.
2. Profile with `perf` (Linux), `Instruments` (macOS), and `dhat` (heap) to
   identify allocation hot spots. Focus on the tokenize-resolve-notify pipeline.
3. For 500+ rule configs, ensure the trie construction is O(n * k) where n is
   rule count and k is average scope depth. Cache the constructed trie in
   `ScopeMatchEngine` across tokenizations of the same document.
4. Add a `.github/workflows/bench.yml` that runs benchmarks on every PR and
   compares against the main branch baseline using criterion's built-in
   comparison reporting.

**Dependencies and Risks:**

- Dependencies: criterion (already in use), dhat, perf tools
- Risk: Benchmark results may be noisy in CI due to shared hardware. Mitigation:
   use criterion's noise detection and set appropriate significance thresholds
- Risk: Some optimizations may increase code complexity. Require that each
   optimization includes a benchmark justification comment

**Success Criteria:**

- Scope resolution: <1us per token at 500 rules, <10us at 1000 rules
- Full document processing: <1ms per 10,000 tokens at 200 rules, <5ms at 500 rules
- Config reload: <10ms for 500-rule config
- Memory: <50MB RSS for a 50,000-line file with 200 rules
- CI benchmark comparison runs on every PR and reports delta

**Estimated Timeline:** 3-4 weeks

**Complexity:** 3/5

---

### v0.15 -- Editor Extension Testing

**Goal:** Establish automated test suites for the VSCode and Neovim editor
extensions, ensuring that changes to core crates don't break editor integrations.

**Depends on:** v0.12 (LSP hardening -- stable protocol behavior required for
reliable integration testing)

**Key Deliverables:**

- VSCode extension test suite:
  - Unit tests for TOML config parsing, font rule generation, decoration
    application
  - Integration tests using VSCode's extension test API (`vscode-test`)
  - End-to-end test: open a Rust file, verify font assignments are applied
- Neovim plugin test suite:
  - Unit tests via busted (Lua testing framework)
  - Integration tests using plenary.nvim test harness
  - Verify highlight groups are set correctly for each scope
- Smoke test matrix: each editor extension tested against the latest polyfont-lsp
  binary from CI
- Test infrastructure documented in CONTRIBUTING.md

**Technical Approach:**

1. For VSCode: add `editors/vscode/src/test/` with Mocha-based tests. Use
   `vscode-test` (the `@vscode/test-electron` package) to launch a VSCode
   instance in test mode. Tests open a fixture file, wait for font assignments,
   and verify decorations via the VSCode API.
2. For Neovim: add `editors/neovim/test/` with busted-style tests. Use
   `plenary.nvim`'s `plenary.test_harness` to test the Lua modules in isolation.
   Integration tests launch a headless Neovim instance, load the plugin, open a
   fixture file, and assert on highlight group state.
3. Create fixture files: a set of source files in multiple languages with known
   expected font assignments. These fixtures are shared across editor test suites.
4. Add editor test steps to CI: VSCode tests run on Linux and macOS (requires
   display server or virtual framebuffer), Neovim tests run on all platforms.

**Dependencies and Risks:**

- Dependencies: vscode-test, busted/plenary.nvim, Xvfb (for headless VSCode tests)
- Risk: VSCode extension test API may change between versions. Pin the test
   VSCode version and update quarterly
- Risk: Neovim headless mode may not fully simulate GUI behavior. Document
   which tests require a GUI and skip them in CI

**Success Criteria:**

- VSCode extension has 20+ tests covering config parsing, rule generation,
  and decoration application
- Neovim plugin has 15+ tests covering highlight setup, scope mapping, and
  commands
- Both test suites run in CI in under 5 minutes
- No regressions in editor extensions when core crates change (verified by CI)

**Estimated Timeline:** 2-3 weeks

**Complexity:** 2/5

---

### v0.16 -- crates.io Publication

**Goal:** Publish all 9 crates to crates.io with complete metadata, semver
stability guarantees, and reproducible builds.

**Depends on:** v0.14 (performance validation -- APIs should be stable before
publication)

**Key Deliverables:**

- All 9 crates published to crates.io with:
  - Complete metadata (description, keywords, categories, license, repository,
    documentation links)
  - `cargo doc` output published to docs.rs without warnings
  - README.md per crate with usage examples
- Semver stability policy enforced:
  - `cargo-semver-checks` passes against the last published version
  - Public API surface documented and reviewed
  - No `#[doc(hidden)]` items in public API that users might depend on
- `publish.sh` updated for the full 9-crate publish order (respecting dependency DAG):
  ```
  polyfont-core -> polyfont-scope -> polyfont-config -> polyfont-parse ->
  polyfont-fonts -> polyfont-themes -> polyfont-lsp -> polyfont-cli ->
  polyfont-render
  ```
- Verify `cargo install polyfont-cli` works from crates.io on all platforms
- Crate download smoke test: create a minimal project depending on each library
  crate and verify it compiles

**Technical Approach:**

1. Audit each crate's `Cargo.toml` for complete metadata. Add `documentation`,
   `readme`, `keywords`, and `categories` fields.
2. Run `cargo doc --workspace --no-deps` and fix all warnings. Ensure all public
   items have doc comments with examples where appropriate.
3. Run `cargo-semver-checks` against a baseline. If breaking changes are
   detected, either fix them or bump the major version (but we're still pre-1.0,
   so breaking changes are acceptable if documented).
4. Test the publish flow with `cargo publish --dry-run` for each crate in
   dependency order. Fix any packaging issues (missing files, unnecessary
   includes).
5. Create a ` crates.io ` publish checklist in CONTRIBUTING.md.

**Dependencies and Risks:**

- Dependencies: crates.io accounts, API tokens in CI secrets
- Risk: Crate name collisions on crates.io. Reserve names early by publishing
   placeholder versions if needed
- Risk: Feature-gated dependencies (tree-sitter grammars, wgpu) may cause
   compilation issues on crates.io's index builders. Test with `cargo publish
   --all-features --dry-run`

**Success Criteria:**

- `cargo install polyfont-cli` installs a working CLI from crates.io
- `cargo add polyfont-core` works and the crate compiles with default features
- All crate docs render on docs.rs without warnings
- `publish.sh --dry-run` succeeds for all 9 crates in CI

**Estimated Timeline:** 2-3 weeks

**Complexity:** 2/5

---

### v0.17 -- Security Audit

**Goal:** Audit the dependency tree and supply chain for vulnerabilities before
the v1.0 production release.

**Depends on:** v0.16 (crates.io publication -- audit against the actual published
dependency tree)

**Key Deliverables:**

- `cargo audit` report: zero known vulnerabilities in the dependency tree
- `cargo deny` configuration: license compliance, banned crates, advisory
  monitoring
- Dependency tree review: identify and minimize transitive dependencies
- Supply chain security:
  - Pin all dependencies to exact versions in `Cargo.lock`
  - Verify integrity of tree-sitter grammar C libraries (checksum validation)
  - Document the provenance of all native dependencies
- Security policy document: vulnerability reporting process, disclosure timeline,
  supported versions
- SBOM (Software Bill of Materials) generated and included in releases

**Technical Approach:**

1. Run `cargo audit` and fix all reported vulnerabilities. If a fix requires a
   dependency upgrade that introduces breaking changes, assess the impact and
   create a dedicated PR.
2. Add `cargo deny` configuration (`.cargo-deny.toml`) with:
   - License whitelist: Apache-2.0, MIT, BSD-2-Clause, BSD-3-Clause, MPL-2.0,
     ISC, CC0-1.0, Unicode-DFS-2016
   - Advisory monitoring: deny for vulnerabilities, warn for unmaintained crates
   - Source monitoring: only allow crates.io and the workspace itself
3. Review the dependency tree with `cargo tree --duplicates`. Eliminate duplicate
   major versions where possible (e.g., multiple versions of `syn` or `serde`).
4. For tree-sitter grammars (compiled C libraries), verify that the source
   commits match the tagged releases and document the build process in a
   `SECURITY.md` file.
5. Generate SBOM using `cargo-cyclonedx` and include it in GitHub Releases.

**Dependencies and Risks:**

- Dependencies: cargo-audit, cargo-deny, cargo-cyclonedx
- Risk: Unfixable vulnerabilities in transitive dependencies (e.g., a crate
   that's unmaintained but has no alternative). Mitigation: document the risk,
   set up monitoring, and plan a migration path
- Risk: License incompatibility in a transitive dependency. Mitigation: cargo-deny
   catches this; address by finding an alternative crate

**Success Criteria:**

- `cargo audit` reports zero vulnerabilities
- `cargo deny check` passes (licenses, advisories, sources)
- Dependency tree has no duplicate major versions (or duplicates are documented
  with justification)
- SBOM is generated and included in the GitHub Release artifacts
- `SECURITY.md` documents the vulnerability reporting process

**Estimated Timeline:** 2 weeks

**Complexity:** 2/5

---

### v1.0 -- Production Release

**Goal:** Ship polyfont v1.0 as a stable, well-documented, production-ready
tool suitable for daily use across all supported editors.

**Depends on:** v0.17 (all previous milestones complete)

**Key Deliverables:**

- API stability: all public crate APIs are frozen; no breaking changes without
  v2.0
- Documentation freeze: user guide, API docs, architecture doc, migration guide
  all reviewed and complete
- Release artifacts:
  - GitHub Release with prebuilt binaries for linux-x64, linux-aarch64,
    macos-x64, macos-aarch64, windows-x64
  - SHA-256 checksums and SBOM for all artifacts
  - All 9 crates published to crates.io
  - VSCode extension published to Marketplace and Open VSX
  - Neovim plugin published to lua.rocks
- CHANGELOG.md reviewed and complete for v1.0
- Release blog post or announcement

**Technical Approach:**

1. Create a `v1.0-rc.1` release candidate tag. Run the full CI pipeline,
   including new benchmarks and editor tests. Invite early adopters to test
   the RC.
2. Fix any issues found during RC testing. Iterate to `v1.0-rc.2` if needed.
3. Update all documentation for the v1.0 release:
   - README.md: update version badges, screenshots, compatibility matrix
   - CONTRIBUTING.md: finalize stability policy for post-1.0
   - docs/architecture.md: verify it reflects the final crate structure
   - docs/migration.md: add v0.10-to-v1.0 migration notes
4. Run the release process:
   - Tag `v1.0.0`
   - CI builds and publishes all artifacts
   - Verify crates.io, VSCode Marketplace, and lua.rocks listings
5. Write a release announcement summarizing the project journey, features, and
   contributor credits.

**Dependencies and Risks:**

- Dependencies: all previous milestones complete
- Risk: RC testing reveals a critical bug that requires a significant fix.
   Mitigation: budget 1-2 weeks of buffer after the RC
- Risk: Marketplace review delays. Mitigation: submit the VSCode extension
   for review during the RC period

**Success Criteria:**

- `cargo install polyfont-cli@1.0.0` works on Linux, macOS, and Windows
- VSCode extension is one-click installable from the Marketplace
- All crate APIs are documented on docs.rs without warnings
- CHANGELOG.md covers every release from v0.1.0 to v1.0.0
- No known vulnerabilities in the dependency tree
- Benchmark baselines recorded for future regression detection

**Estimated Timeline:** 2-3 weeks (including RC period)

**Complexity:** 2/5

---

## Pre-v1.0 Timeline Summary

| Milestone | Version | Complexity | Duration | Dependencies |
|-----------|---------|-----------|----------|--------------|
| Render Engine Completion | v0.11 | 4/5 | 4-5 weeks | None (parallel) |
| LSP Production Hardening | v0.12 | 3/5 | 3-4 weeks | None (parallel) |
| Neovim Native Font Support | v0.13 | 4/5 | 5-8 weeks | None (parallel) |
| Performance at Scale | v0.14 | 3/5 | 3-4 weeks | v0.12 |
| Editor Extension Testing | v0.15 | 2/5 | 2-3 weeks | v0.12 |
| crates.io Publication | v0.16 | 2/5 | 2-3 weeks | v0.14 |
| Security Audit | v0.17 | 2/5 | 2 weeks | v0.16 |
| Production Release | v1.0 | 2/5 | 2-3 weeks | v0.17 |

**Critical path:** v0.12 -> v0.14 -> v0.16 -> v0.17 -> v1.0 (12-16 weeks)

**Parallel tracks:** v0.11, v0.13, and v0.15 can proceed concurrently with the
critical path.

**Estimated total to v1.0:** 16-24 weeks from v0.10, depending on parallelism
and upstream review timelines (v0.13 Neovim RFC).

---

## Post-v1.0 Roadmap

These items represent the long-term direction. Each is a candidate for a minor
or major version bump depending on scope. None are committed to a specific
timeline.

---

### Collaborative Theme Sharing

**Goal:** Enable teams and communities to share polyfont configurations through
a distributed, gist-based registry.

**Key Deliverables:**

- `polyfont theme share` CLI command: publishes the current config as a GitHub
  Gist or to a community registry URL
- `polyfont theme install <gist-url>`: fetches and installs a shared theme
- Community theme registry: a JSON index (hosted on GitHub Pages or a dedicated
  repo) listing user-submitted themes with metadata
- Theme versioning: semantic version tracking in shared themes with upgrade
  notifications
- Team sharing: `.polyfont/themes/` directory in project repos for team-wide
  config distribution

**Technical Approach:**

Extend the existing `ThemeShare` and `ThemeLockfile` types in polyfont-themes.
Add a `ThemeRegistryClient` that fetches and caches remote themes. For Gist-based
sharing, use the GitHub Gists API. For the community registry, use a static JSON
file updated via PR.

**Complexity:** 2/5

---

### Custom Rendering Plugin System

**Goal:** Allow third-party rendering backends via a plugin API, enabling
custom text effects (animations, gradient fonts, 3D extrusion) beyond the
built-in GPU and software renderers.

**Key Deliverables:**

- `RenderPlugin` trait in polyfont-render with lifecycle hooks:
  `init()`, `render_line()`, `render_document()`, `shutdown()`
- Plugin discovery: load render plugins from `~/.config/polyfont/plugins/` or
  a configurable path
- WASM plugin support: compile plugins to WASM for sandboxed execution
- Example plugins: debug overlay, font metrics visualization, accessibility
  highlight modes

**Technical Approach:**

Define the `RenderPlugin` trait as a stable ABI (using `abi_stable` or WASM
imports/exports). The `RenderEngine` maintains a plugin chain and calls each
plugin's hooks in order. WASM plugins are loaded via `wasmtime` with a
restricted capability set (no filesystem access, no network).

**Complexity:** 4/5

---

### Variable Font Axes Support in Editors

**Goal:** Extend editor integrations to control variable font axes (weight,
width, slant, optical size) per token, building on the `AxisValue` types
already in polyfont-core.

**Key Deliverables:**

- VSCode extension: generate CSS `font-variation-settings` values in decorations
- Neovim plugin: expose axis values in highlight metadata for GUI consumption
- CLI: `polyfont dump --format=css` outputs `font-variation-settings` per scope
- Validation: warn when axis values are outside the font's declared range

**Technical Approach:**

The `FontSpec.axes` field and `css_variation_settings()` method already exist
in polyfont-core. The work is in the editor extensions: map `AxisValue` to the
editor's font API. For VSCode, use the CSS font-variation-settings property in
webview-based rendering. For native editors, use the platform's variable font
API (FreeType/HarfBuzz, DirectWrite, CoreText).

**Complexity:** 3/5

---

### IDE-Agnostic Font Preview

**Goal:** Provide a universal font preview tool that shows how a polyfont
configuration renders across multiple languages and themes, independent of any
specific editor.

**Key Deliverables:**

- `polyfont preview` CLI command: renders a multi-language sample file using
  the current config and outputs PNG/SVG/HTML
- Web-based preview: generate a static HTML page with CSS font-variation-settings
  that renders the sample in a browser
- Side-by-side comparison: preview multiple configs or themes simultaneously
- Integration with `polyfont check`: preview shows which tokens are matched and
  which fall through to defaults

**Technical Approach:**

Build on the completed render engine (v0.11). The CLI command reuses the
`RenderEngine` to produce output. The web preview generates an HTML file with
`@font-face` declarations and per-span `font-family` + `font-variation-settings`
CSS, producing a zero-dependency preview page.

**Complexity:** 2/5

---

### Font Pairing AI/ML Suggestions

**Goal:** Use machine learning to suggest font pairings based on typographic
principles, existing popular configurations, and user preferences.

**Key Deliverables:**

- Font pairing model: trained on popular monospace font combinations and
  typographic contrast principles (serif vs sans-serif, weight contrast,
  x-height compatibility)
- `polyfont suggest` CLI command: analyzes the user's scope distribution and
  suggests font pairings
- LSP integration: code action that suggests font assignments for unscoped tokens
- Feedback loop: users can rate suggestions, improving the model over time

**Technical Approach:**

Start with a rule-based system using curated font pairing heuristics (the
existing `FONT_PAIRINGS` database in polyfont-lsp). For ML, train a lightweight
model (potentially a decision tree or small neural network) on font metadata
features (weight range, width, x-height, contrast) to predict complementary
pairings. Ship the model as a static asset in the crate.

**Complexity:** 4/5

---

### Enterprise Features

**Goal:** Support organizational deployment with centralized configuration
management and team sharing.

**Key Deliverables:**

- Centralized config server: `polyfont serve` runs a lightweight HTTP server
  that distributes configs to team members
- Config management: hierarchical configs (org-level, team-level, project-level,
  user-level) with merge policies
- Audit logging: track config changes and font assignment overrides
- SSO integration: optional authentication for the config server via OAuth2/OIDC
- Deployment tooling: Ansible/Chef/Terraform modules for polyfont deployment

**Technical Approach:**

Add a `polyfont-serve` binary (or extend polyfont-cli with a `serve` subcommand)
using `axum` or `actix-web`. The server hosts configs at predictable URLs
(`/config/{org}/{team}/{project}`). The `ConfigLoader` gains a remote fetch mode
that queries the server. Merge policies follow the existing hierarchical merge
logic in polyfont-config.

**Complexity:** 3/5

---

### Additional Language Grammars

**Goal:** Expand tree-sitter language support beyond the current 10 languages.

**Key Deliverables:**

- Priority grammars: Haskell, Scala, Kotlin, Swift, Elixir, OCaml, R, SQL,
  Shell/Bash, TOML, YAML, JSON, Dockerfile
- Grammar quality validation: each grammar must achieve >90% token coverage on
  a representative corpus of files
- Community grammar contributions: documented process for adding new grammars
- Grammar bundle: optional feature flag that includes all grammars in a single
  build

**Technical Approach:**

Each grammar follows the existing pattern: add a `tree-sitter-*` dependency to
polyfont-parse behind a feature gate, implement the tokenizer, add highlight
queries. Prioritize based on user demand and grammar quality.

**Complexity:** 2/5 per language

---

### WebAssembly Rendering for Browser-Based Editors

**Goal:** Enable polyfont rendering in browser-based editors (VSCode Web,
GitHub.dev, Zed Web, code-server) via WebAssembly.

**Key Deliverables:**

- `polyfont-render` compiled to WASM with canvas-based rendering
- Browser API: JavaScript/TypeScript wrapper that accepts font assignments and
  renders annotated code in a `<canvas>` element
- VSCode Web extension: polyfont extension that works in github.dev and
  vscode.dev
- Performance: rendering latency <16ms per frame (60fps target)

**Technical Approach:**

Compile polyfont-core, polyfont-scope, and polyfont-render to WASM using
`wasm-pack`. The browser module uses the Canvas API for rendering and the
Web Font API for loading fonts. For VSCode Web, register as a web extension
using the VSCode Web Extension API. Font files are loaded via `@font-face`
CSS declarations.

**Complexity:** 4/5

---

## Post-v1.0 Summary

| Feature | Complexity | Priority | Major Version |
|---------|-----------|----------|---------------|
| Collaborative Theme Sharing | 2/5 | High | v1.x |
| IDE-Agnostic Font Preview | 2/5 | High | v1.x |
| Additional Language Grammars | 2/5 each | High | v1.x |
| Variable Font Axes in Editors | 3/5 | Medium | v1.x |
| Enterprise Features | 3/5 | Medium | v2.x |
| Custom Rendering Plugins | 4/5 | Medium | v2.x |
| Font Pairing AI/ML | 4/5 | Low | v2.x |
| WebAssembly Rendering | 4/5 | Medium | v2.x |

---

## Decision Log

This section records significant architectural decisions. Entries are appended
as decisions are made.

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-20 | Use tower-lsp for LSP server | Mature, async, well-maintained Rust LSP framework |
| 2026-05-20 | TOML for config format | Human-readable, native Rust support, popular in Rust ecosystem |
| 2026-05-20 | TextMate scopes for token classification | Universal syntax highlighting standard, supported by all target editors |
| 2026-05-20 | Per-crate workspace layout | Clean separation of concerns, independent publishable units |
| 2026-05-21 | Feature-gated tree-sitter grammars | Avoids heavy C dependencies for users who don't need parsing |
| 2026-05-21 | TrieScopeResolver for O(k) scope lookup | Scales to large rule sets without O(n) linear scan |
| 2026-05-21 | Built-in themes with prefix-match fallback | Themes work even with incomplete scope coverage |
| 2026-05-21 | Kitty symbol_map for Helix | Terminal cannot do per-scope fonts; Unicode range mapping is the best available approximation |
| 2026-05-25 | GPU + software dual render path | Not all environments have GPU access; software fallback ensures universal compatibility |
| 2026-05-25 | Gist-based theme sharing | Lowest-friction sharing mechanism; no server infrastructure required |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code style, testing,
and release process.
