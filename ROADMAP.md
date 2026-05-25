# Polyfont Roadmap

Per-token font highlighting for code editors. This document tracks milestones from the
current v0.1.0 prototype through v1.0 and beyond.

**Current version:** 0.10.0 (2026-05-25)
**Repository:** https://github.com/WyattAu/polyfont
**License:** Apache-2.0

---

## Project Structure Reference

```
crates/
  polyfont-core/     # TokenInfo, FontSpec, FontRule, ScopeMatchEngine, benchmarks
  polyfont-config/   # TOML config loading, PolyfontConfig, ConfigLoader
  polyfont-scope/    # ScopePattern, ScopeSelector, ScopeResolver, TrieScopeResolver, ScopeTree
  polyfont-lsp/      # tower-lsp server, tree-sitter-first tokenizer, naive fallback
  polyfont-cli/      # check, vscode, neovim, kitty, dump, font, theme subcommands
  polyfont-parse/    # Tree-sitter token parser, 10 language grammars, naive fallback
  polyfont-fonts/    # Cross-platform font discovery, download, caching (Linux/macOS/Windows)
  polyfont-themes/   # Theme import/export, 5 built-in themes, accessibility, collaborative
  polyfont-render/   # Custom multi-font rendering engine skeleton (GPU + software paths)
editors/
  vscode/            # VSCode extension (textMateRules fontFamily API)
  neovim/            # Lua plugin with nvim-treesitter, font metadata for GUIs
  zed/               # Zed extension (Wasm-based, text_style_overrides)
  helix/             # Integration guide for Helix (Kitty symbol_map approximation)
  sublime/           # Sublime Text Python plugin (.sublime-theme generation)
  jetbrains/         # JetBrains IntelliJ plugin skeleton (Kotlin)
```

---

## v0.2 -- Tree-sitter LSP Integration

**Goal:** Replace the naive line-prefix tokenizer in `polyfont-lsp` with actual
Tree-sitter parsing to produce correct, per-token scope classifications.

### Key Deliverables

- Tree-sitter-based tokenizer replacing `tokenize_document()` and `classify_line()` in
  `crates/polyfont-lsp/src/lib.rs:251-340`
- `tree-sitter-highlight` integration mapping capture names to TextMate-compatible scope
  strings (e.g., `keyword.control`, `entity.name.function`)
- Per-language grammars bundled or fetched at runtime: Rust, TypeScript, Python, Go
- Incremental re-parsing on `textDocument/didChange` using tree-sitter's edit API
- Grammar auto-detection from file extension and language ID in the LSP initialization
  parameters
- Updated `TokenInfo` structs with full per-token ranges (current line-prefix approach
  only produces one token per non-blank line)

### Technical Approach

1. Introduce a new crate `polyfont-parse` (or expand `polyfont-core`) that wraps
   `tree-sitter` and `tree-sitter-highlight`. This crate owns the parser lifecycle,
   grammar loading, and scope emission.

2. Replace the `tokenize_document` free function in `polyfont-lsp/src/lib.rs` with a
   call into the new parser. The parser accepts `(text, language_id) -> Vec<TokenInfo>`
   where each `TokenInfo` has a precise byte range and a dot-separated scope string
   derived from the tree-sitter highlight capture.

3. For incremental parsing, maintain a `HashMap<String, ParserState>` in `ServerState`
   keyed by document URI. On `didChange`, compute the tree-sitter `InputEdit` from the
   LSP content change (position deltas), apply it to the existing tree, and re-highlight
   only the changed region. This avoids full re-parse on every keystroke.

4. Grammar distribution: ship compiled `.so`/`.dylib`/`.dll` grammar libraries for the
   four initial languages via a `polyfont-grammars` optional dependency or download them
   on first use into a cache directory (`~/.cache/polyfont/grammars/`). The download
   approach keeps the base binary small but requires network access; the bundled
   approach is simpler for offline use. Decision: start bundled, add download later
   (see v0.4).

5. Scope mapping: `tree-sitter-highlight` emits capture names like `"keyword"`,
   `"string"`, `"function"`, etc. Map these to TextMate scopes using a per-language
   query file (e.g., `queries/rust/highlights.scm`). Leverage the existing query files
   from nvim-treesitter or tree-sitter itself.

### Dependencies

- `tree-sitter` >= 0.24 (Rust bindings)
- `tree-sitter-highlight` >= 0.24
- Per-language grammar crates: `tree-sitter-rust`, `tree-sitter-typescript`,
  `tree-sitter-python`, `tree-sitter-go`
- Updated `TokenInfo` in `crates/polyfont-core/src/token.rs` -- no breaking changes
  expected, the struct already has the right fields

### Estimated Complexity: L (Large)

The incremental parsing logic and grammar distribution are the primary sources of
complexity. The tree-sitter API itself is well-documented, but mapping its highlight
captures to TextMate scopes in a consistent, language-agnostic way requires careful
query design.

### Risks

- **Grammar compilation across platforms:** tree-sitter grammars are C libraries that
  must be compiled per-target. CI must produce artifacts for linux-x64, macos-aarch64,
  windows-x64 at minimum. Consider using `cc` crate with pre-built artifacts.
- **Incremental edit accuracy:** LSP `textDocument/didChange` sends position-based
  ranges; converting these to tree-sitter byte-offset `InputEdit` requires careful
  UTF-8 handling since LSP positions are UTF-16 code unit based.
- **Scope coverage:** tree-sitter highlight queries may not cover all syntactic
  constructs for every language. Gaps produce unscoped tokens that fall through to the
  catch-all rule. This is acceptable for v0.2 but must be documented.

### Success Criteria

- `polyfont-lsp` produces correct per-token font assignments for Rust, TypeScript,
  Python, and Go source files with >95% token coverage (excluding whitespace and
  plain text)
- Incremental re-parse on single-character edits completes in <5ms for files up to
  10,000 lines
- All 38 existing tests continue to pass
- New tests cover: grammar loading, scope mapping for each language, incremental edit
  application, graceful fallback when grammar is missing
- CI builds grammar artifacts for all three platforms

### Estimated Timeline

4-6 weeks

---

## v0.3 -- Neovim GUI Native Support

**Goal:** Enable per-token font rendering in Neovim GUI frontends by extending
highlight attributes with font family information.

### Key Deliverables

- RFC submitted to neovim/neovim proposing a `font` field in `nvim_set_hl()` highlight
  attributes (similar to the existing `gui` field for guifont)
- Working integration with Goneovim (Go-based GUI, most capable for custom rendering)
- Working font-rendering patch for Neovide (Rust + Skia)
- Compatibility test matrix across Neovim GUIs: Goneovim, Neovide, VimR, nvui,
  code-minimap (if applicable)
- Fallback mechanism: when GUI does not support the `font` attribute, the plugin
  silently degrades (current behavior)

### Technical Approach

1. **Neovim core change:** Submit a PR to `neovim/neovim` adding an optional `font`
   string field to highlight definitions. The change is small in scope -- the
   `hlstate` in `src/nvim/highlight_group.c` gains a `font_name` field, and each
   GUI renderer reads it when constructing `attrs`. This is a non-breaking additive
   change; existing frontends ignore the field.

2. **Goneovim integration:** Goneovim renders via Go's `github.com/mattn/go-gtk` and
   custom OpenGL shaders. The highlight system already reads per-highlight-group
   attributes. Add font family selection per highlight group by maintaining a font
   cache (Pango font descriptions keyed by family name) and switching fonts during
   text layout.

3. **Neovide integration:** Neovide uses `skribo` (a Rust font shaping library built
   on `fontdb`/`glyphon`) for text rendering. Each highlight group already maps to a
   set of attributes. Patch the highlight application to include font family in the
   font ID lookup. The main challenge is that `skribo` creates font collections at
  startup; per-token font switching requires either a font collection per highlight
  group or lazy font loading.

4. **Plugin changes:** Update `editors/neovim/lua/polyfont/highlights.lua` to set the
   `font` attribute instead of (or in addition to) the current workaround. The
   Tree-sitter integration in `editors/neovim/lua/polyfont/treesitter.lua` already
  maps captures to scope names, so the highlight application path only needs the
   new attribute.

### Dependencies

- Neovim RFC acceptance (unknown timeline -- this is an upstream project decision)
- Goneovim maintainers willing to review/merge changes
- Neovide maintainers willing to review/merge changes
- `editors/neovim/lua/polyfont/` -- existing plugin code, no structural changes needed

### Estimated Complexity: L (Large)

Upstream coordination with three separate projects (neovim, Goneovim, Neovide) adds
process complexity. The technical changes themselves are moderate.

### Risks

- **RFC rejection or indefinite deferral:** The neovim team may not prioritize this
  feature. Mitigation: implement as a Neovim-only patch set that can be maintained
  out-of-tree until accepted.
- **Font rendering performance in Neovide:** Per-token font switching in Skia may be
  expensive if fonts are not cached. Benchmark early.
- **Goneovim maintenance status:** Goneovim has had periods of low activity. Verify
  current maintainer availability before investing effort.

### Success Criteria

- RFC filed with neovim with clear API proposal and backward compatibility argument
- At least one GUI frontend (Goneovim or Neovide) renders distinct fonts per scope
  in a demo configuration
- The existing Neovim plugin continues to work with terminal Neovim (no regression)
- Documentation describes which GUIs are supported and how to enable font highlighting

### Estimated Timeline

6-10 weeks (includes upstream review cycles)

---

## v0.4 -- Font Management

**Goal:** Automate font discovery, downloading, caching, and verification so users do
not need to manually install fonts or know their system font paths.

### Key Deliverables

- `polyfont font install <family>` CLI subcommand that downloads fonts from GitHub
  releases or Google Fonts API
- `polyfont font list` subcommand showing all installed and available fonts
- `polyfont font preview <family>` subcommand that renders a sample code snippet in
  the specified font (using a terminal-friendly fallback or outputting a small SVG)
- Cross-platform font discovery:
  - Linux: `fc-list` invocation via `std::process::Command`
  - macOS: `CTFontManager` via `core-text` crate or `fc-list` fallback
  - Windows: Windows Registry `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts`
    via `winreg` crate
- Font caching in `~/.cache/polyfont/fonts/` with SHA-256 deduplication
- Config validation enhancement: `polyfont check` verifies that every referenced font
  family is discoverable on the current system and suggests installation if missing

### Technical Approach

1. Create a new crate `polyfont-fonts` that encapsulates all font management logic.
   Depends on platform-specific crates (`fontconfig` on Linux, `core-text` on macOS,
   `winreg` on Windows) behind a `FontDiscovery` trait with per-platform
   implementations selected via `#[cfg(target_os)]`.

2. Font download: for GitHub-sourced fonts (e.g., JetBrains Mono, Fira Code), resolve
   the latest release via the GitHub API, download the appropriate asset (zip of OTF/TTF
   files), extract, and copy into the cache directory. For Google Fonts, use the
   Google Fonts API to fetch the font file URL. Verify downloads with SHA-256 checksums
   stored in a lockfile (`~/.cache/polyfont/fonts.lock.toml`).

3. Font discovery: the `FontDiscovery::list_families()` method returns `Vec<String>` of
   installed font family names. The `FontDiscovery::find_family(&str) -> Option<FontPath>`
   method returns the filesystem path to the font file. These replace the naive
   `check_font_available()` and `scan_for_font()` functions in
   `crates/polyfont-cli/src/main.rs:85-133`.

4. Integration with `polyfont-config`: add an optional `[[fonts.sources]]` section to
   `.polyfont.toml` that declares where fonts should be sourced from, allowing
   reproducible font setups across machines.

### Dependencies

- New crate: `polyfont-fonts`
- `reqwest` for HTTP downloads (with optional `rustls-tls` feature to avoid OpenSSL)
- `sha2` for checksum verification
- `zip` for extracting downloaded font archives
- Platform crates: `fontconfig-sys` (Linux), `core-text` (macOS), `winreg` (Windows)
- `fontdb` (from the `parley`/`cosmic-text` ecosystem) as a cross-platform font
  database alternative to raw platform APIs

### Estimated Complexity: M (Medium)

The per-platform discovery is tedious but straightforward. Download and cache logic
is well-trodden territory. The main uncertainty is handling font family name
normalization across platforms (e.g., "Fira Code" vs "FiraCode" vs "firacode").

### Risks

- **Font licensing:** Some fonts restrict redistribution. The download command must
  verify license compatibility before caching. At minimum, display the license type
  and require user confirmation for non-OFL/MIT fonts.
- **Font family name normalization:** Different platforms report family names
  differently. "JetBrains Mono" may appear as "JetBrains Mono NL", "JetBrainsMono",
  or with weight suffixes. A fuzzy matching layer is needed.
- **Large font files:** Some font families include 20+ weight/style variants totaling
  50MB+. Only download the weights referenced in the user's config.

### Success Criteria

- `polyfont font install JetBrains Mono` downloads and caches the font, and
  `polyfont check` subsequently reports it as available
- `polyfont font list` returns accurate results on Linux, macOS, and Windows
- `polyfont check` suggests installation commands for missing fonts
- No regressions in existing CLI subcommands

### Estimated Timeline

3-4 weeks

---

## v0.5 -- Performance and Scale

**Goal:** Establish a performance baseline and optimize scope matching and token
processing to handle large files with negligible latency.

### Key Deliverables

- Criterion benchmark suite covering:
  - `ScopeMatchEngine::resolve_token()` throughput (tokens/sec)
  - `ScopeResolver::resolve()` throughput
  - `ScopeTree` construction and prefix query
  - Full document tokenization pipeline (end-to-end, text to font assignments)
  - Config loading and engine construction
- O(1) average-case scope lookup replacing the current O(n) linear scan in
  `ScopeMatchEngine::resolve_token()` (`crates/polyfont-core/src/engine.rs:67-79`)
  and `ScopeResolver::resolve()` (`crates/polyfont-scope/src/lib.rs:212-243`)
- Lazy font loading: font files are only read/parsed when a matching token is first
  encountered
- Memory profiling with `dhat` or `jemalloc` profiler; target: <10MB RSS for a
  50,000-line file with 100 rules
- Latency targets:
  - Scope resolution: <1us per token
  - Full document processing (tokenize + resolve): <1ms per 10,000 tokens
  - Config reload: <5ms

### Technical Approach

1. **Hashed trie for scope lookup:** Replace the `Vec<FontRule>` linear scan in
   `ScopeMatchEngine` with a prefix trie where each node stores a reference to the
   best-matching rule at that depth. The existing `ScopeTree` in
   `crates/polyfont-scope/src/lib.rs:268-342` provides the data structure; extend it
   to store rule references and support specificity-aware lookup. On construction,
   insert each rule's scope pattern into the trie. On lookup, traverse the trie from
   root, tracking the most specific match. This reduces lookup from O(rules) to
   O(scope_depth), which is bounded by ~5-6 segments in practice.

2. **Benchmarks:** Create `crates/polyfont-core/benches/` with criterion microbenchmarks.
   Generate synthetic workloads: 10K/50K/100K tokens with 10/50/200 rules. Measure
   throughput and latency distributions. Add a macrobenchmark that processes a real
   10K-line Rust file end-to-end.

3. **Lazy font loading:** The `FontSpec` struct currently stores only metadata (family
   name, weight, style). Actual font file loading (parsing OTF/TTF headers, building
   glyph caches) should be deferred until the editor requests rendering data. This
   primarily affects the LSP server's memory footprint since it holds font assignments
   for all open documents.

4. **Memory optimization:** Profile the `ServerState` struct in `polyfont-lsp`. The
   `documents: HashMap<String, DocumentState>` stores full document text; for large
   workspaces this can grow unbounded. Consider an LRU eviction policy or only storing
   the tree-sitter parse tree (which is more compact than raw text).

### Dependencies

- `criterion` for benchmarking
- `dhat` for heap profiling
- No new crates for the trie optimization -- built on existing `ScopeTree`

### Estimated Complexity: M (Medium)

The trie optimization is the core work; everything else is measurement and incremental
tuning.

### Risks

- **Trie memory overhead:** A trie with 200 rules and deep scope patterns may use more
  memory than a flat vector for small rule sets. Benchmark both approaches and add a
  heuristic: use the trie when rule count exceeds ~20, fall back to linear scan
  otherwise.
- **Latency variance:** Editor responsiveness depends on the full pipeline (LSP
  notification -> tokenization -> scope resolution -> editor rendering). The Rust-side
  latency targets are necessary but not sufficient; editor-side rendering overhead
  may dominate.

### Success Criteria

- Scope resolution benchmarks show >=10x improvement over v0.1 linear scan for
  200-rule configurations
- Criterion reports are generated in CI and trends are tracked
- No memory leaks detected in long-running sessions (verified with `dhat` over
  10,000 document open/close cycles)
- The benchmark suite runs in <60 seconds in CI

### Estimated Timeline

3-4 weeks

---

## v0.6 -- Theme Ecosystem

**Goal:** Enable users to import existing TextMate color themes as polyfont
configurations and share polyfont-specific themes with the community.

### Key Deliverables

- `polyfont theme import <path-or-url>` CLI subcommand that reads a `.tmTheme` XML
  file (TextMate theme format) or a VSCode `.json` theme and generates a
  `.polyfont.toml` with scope-to-font mappings derived from the theme's font settings
- `polyfont theme export` subcommand that outputs the current config as a standalone
  theme file
- Theme registry: a simple JSON index hosted in the repository (or a dedicated GitHub
  repo, `polyfont-themes`) listing community-contributed themes with metadata
  (name, author, font list, scope coverage)
- Import support for at least: One Dark Pro, Dracula, Catppuccin, Tokyo Night,
  Gruvbox, Nord
- Theme validation: verify that all referenced fonts exist and scope patterns are
  syntactically valid
- `polyfont theme apply <name>` that fetches a theme from the registry and writes it
  to `.polyfont.toml`

### Technical Approach

1. **TextMate theme parsing:** `.tmTheme` files are plist XML. Parse with `quick-xml`
  or `plist` crate. Each theme entry maps a scope name to color settings (foreground,
   background, fontStyle). For polyfont import, extract the `fontStyle` field (which
   may specify bold/italic) and map it to a `FontWeight`/`FontStyle` in the
   polyfont config. Font family is not specified in TextMate themes (they only define
   colors), so the importer must prompt the user or use a default mapping strategy.

2. **VSCode theme parsing:** VSCode themes are JSON with a `tokenColors` array of
   `{scope, settings: {foreground, fontStyle, ...}}` entries. Same approach as above
   for fontStyle extraction.

3. **Theme-to-font mapping strategy:** Since TextMate themes do not specify font
   families, the importer uses a configurable mapping file that assigns font families
   to scope patterns. For example:
   ```toml
   [mapping]
   "keyword" = { family = "Maple Mono", weight = "bold" }
   "comment" = { family = "IBM Plex Mono", style = "italic" }
   ```
   The user can provide their own mapping or use the default. This mapping file is
   versioned alongside the theme.

4. **Registry format:** A `themes.json` file in the repository root:
   ```json
   {
     "themes": [
       {
         "name": "monaspace-dark",
         "author": "...",
         "source": "https://github.com/.../theme.polyfont.toml",
         "fonts": ["Monaspace Argon", "Monaspace Neon", "..."],
         "scope_count": 18
       }
     ]
   }
   ```
   Themes are fetched via HTTP and cached locally.

### Dependencies

- `quick-xml` or `serde-xml-rs` for plist parsing
- `reqwest` for theme fetching (already needed for v0.4 font downloads)
- No new crate dependencies beyond what v0.4 introduces

### Estimated Complexity: M (Medium)

Parsing existing theme formats is well-defined work. The main challenge is the
theme-to-font mapping strategy, which requires design decisions that affect the user
experience.

### Risks

- **Theme quality varies:** Some TextMate themes have inconsistent scope naming or
   very broad selectors (`*` with a single color). The imported polyfont config may
   not produce visually distinct results. Document this limitation.
- **Font licensing in themes:** A theme that references proprietary fonts cannot be
   shared freely. The registry must track font licensing and warn users.

### Success Criteria

- `polyfont theme import onedark.tmTheme` produces a valid `.polyfont.toml` without
  errors
- At least 5 popular themes are importable and produce visually correct results
- `polyfont theme apply monaspace-dark` downloads and applies a registry theme
- Theme validation catches invalid scope patterns and missing fonts

### Estimated Timeline

3-4 weeks

---

## v0.7 -- Additional Editor Support

**Goal:** Expand polyfont support to Zed, Helix, JetBrains IDEs, and Sublime Text.

### Key Deliverables

- **Zed editor extension:** Leverage Zed's GPUI rendering engine, which supports
  per-highlight font family assignment. Create a Zed extension that reads
  `.polyfont.toml` and applies font rules via Zed's theme system.
- **Helix workaround:** Helix runs in the terminal and does not support per-token
  font switching natively. Generate Kitty `symbol_map` configuration that maps
  specific Unicode ranges to fonts, approximating per-scope font differentiation for
  Kitty terminal users.
- **JetBrains IDE plugin:** Kotlin plugin for IntelliJ Platform that intercepts
  syntax highlighting and applies font family overrides via the editor's styling API.
- **Sublime Text package:** `.sublime-package` that reads `.polyfont.toml` and generates
  `.sublime-theme` overrides with per-scope font settings.

### Technical Approach

1. **Zed:** Zed's `Theme` settings support `font_family` per syntax scope in recent
   versions. The extension reads the polyfont config and generates Zed theme JSON
   entries. The main integration point is Zed's extension API for custom themes.
   Verify that Zed's `text_style_overrides` in `settings.json` supports per-scope
   font families (this is a newer feature and may have limitations).

2. **Helix/Kitty:** Kitty's `symbol_map` maps Unicode code point ranges to fonts.
   This cannot achieve true per-scope font switching (since the same character can
   appear in different scopes), but it can approximate it for distinctive character
   sets. For example, map mathematical operators (U+2200-U+22FF) to one font and
   box-drawing characters (U+2500-U+257F) to another. Generate the `symbol_map`
   entries from the polyfont config's scope-to-font rules, mapping scopes to their
   characteristic Unicode ranges.

3. **JetBrains:** Use the IntelliJ Platform SDK to create a plugin that:
   - Listens for editor highlight events
   - Maps PSI element types to polyfont scopes
   - Applies `EditorFontType` overrides via `EditorSettings`
   The JetBrains platform's font handling is relatively rigid (per-editor, not
   per-token), so this may require a custom rendering approach using
   `EditorCustomElementRenderer`.

4. **Sublime Text:** Sublime Text's `.sublime-theme` files support per-scope
   font settings via the `font.face` key in `dict` entries. The package reads
   `.polyfont.toml` and generates a hidden `.sublime-theme` override file that
   maps scopes to fonts.

### Dependencies

- Zed: Zed extension API (Rust or Wasm-based)
- Helix: Kitty terminal (no code changes to Helix itself)
- JetBrains: IntelliJ Platform SDK, Kotlin
- Sublime Text: Python 3 plugin API
- Each editor integration is independent and can be developed in parallel

### Estimated Complexity: L (Large)

Four separate editor integrations, each with different languages, APIs, and
limitations. JetBrains is the most complex due to the platform's rigid font model.

### Risks

- **Zed API limitations:** Zed's per-scope font support may not be granular enough
  for all polyfont use cases. Verify before investing significant effort.
- **JetBrains font model:** JetBrains IDEs apply one font per editor instance, not
   per token. Achieving per-token fonts likely requires a custom renderer, which is
   fragile and may break across IntelliJ versions.
- **Helix approximation quality:** The Kitty `symbol_map` approach is inherently
   limited. Document clearly that this is an approximation, not true per-scope
   rendering.

### Success Criteria

- At least two new editors have working integrations (Zed + one other)
- Each integration has a documented setup guide and example config
- CI builds and tests each integration where possible (JetBrains plugin build,
   Sublime package lint)

### Estimated Timeline

6-8 weeks (parallelizable across editors)

---

## v1.0 -- Production Release

**Goal:** Ship a stable, well-documented, easily installable polyfont release suitable
for daily use.

### Key Deliverables

- **SemVer enforcement:** CI job that checks the workspace version in
  `Cargo.toml:12` matches the git tag and that no public API changes occur without a
  version bump. Use `cargo-semver-checks` to detect breaking changes in crate public
  APIs.
- **Stability guarantees:** No breaking changes to config format, CLI interface, or
  LSP protocol without a major version bump. Document the stability policy in
  `CONTRIBUTING.md`.
- **Comprehensive documentation:**
  - User guide: installation, configuration, editor setup for each supported editor
  - API docs: `cargo doc` for all public crate APIs, published to docs.rs
  - Architecture doc: component diagram, data flow, extension points
  - Migration guide: how to upgrade between minor versions
- **Published crates on crates.io:**
  - `polyfont-core`
  - `polyfont-config`
  - `polyfont-scope`
  - `polyfont-lsp`
  - `polyfont-cli`
  - `polyfont-fonts` (if introduced in v0.4)
  - `polyfont-parse` (if introduced in v0.2)
- **VSCode Marketplace listing:** Published extension with marketplace validation
  passing, README screenshots, and usage instructions
- **Neovim plugin on lua.rocks:** Published as `polyfont.nvim` with rockspec
- **Binary releases:** GitHub Releases with prebuilt binaries for linux-x64,
  linux-aarch64, macos-x64, macos-aarch64, windows-x64. Includes CLI and LSP binaries.
  `cargo install polyfont-cli` works from crates.io.
- **Changelog:** Maintained `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/)
  format with entries for every release

### Technical Approach

1. **Crates.io publishing:** Ensure all crates have complete metadata (description,
   keywords, categories, license, repository, documentation links). Add a
   `publish.sh` script that runs `cargo publish --dry-run` for each crate in dependency
   order, then publishes. Gate on CI passing.

2. **SemVer checking:** Add `cargo-semver-checks` to CI. Compare the current branch's
   public API against the latest published version on crates.io. Fail CI if breaking
   changes are detected without a major version bump.

3. **VSCode Marketplace:** The extension at `editors/vscode/` must pass `vsce ls`
   validation. Generate marketplace-ready README with screenshots (use a headless
   screenshot tool or manual captures). Use `ovsx` for Open VSX Registry publication
   in addition to the proprietary marketplace.

4. **lua.rocks:** Create a rockspec that points to the git tag. The rockspec declares
   dependencies (nvim-treesitter) and build steps (none for a pure-Lua plugin).

5. **Binary releases:** Extend the existing `.github/workflows/ci.yml:58-79` release
   job to produce tarballs/zipfiles for all five targets. Use `cross` for
   cross-compilation where needed. Generate SHA-256 checksums. Attach to GitHub
   Release using `softprops/action-gh-release`.

### Dependencies

- All previous milestones completed and stable
- `cargo-semver-checks` for API diff detection
- `vsce` or `ovsx` for VSCode extension packaging
- `cross` for cross-compilation
- lua.rocks account for plugin publication
- crates.io credentials for crate publication

### Estimated Complexity: M (Medium)

Mostly process and documentation work. The technical foundations are laid by v0.2-v0.7.

### Risks

- **Crates.io naming conflicts:** `polyfont-core` etc. may be taken. Reserve crate
  names early (create empty crates if necessary).
- **VSCode Marketplace review:** Microsoft's marketplace has review requirements
  including privacy policy, icon, screenshots. Budget time for review cycles.
- **lua.rocks infrastructure:** lua.rocks has intermittent availability issues.
  Have a fallback installation method (git clone + `:pack add`).

### Success Criteria

- `cargo install polyfont-cli` installs a working CLI from crates.io
- VSCode extension is installable from the marketplace with one click
- `luarocks install polyfont.nvim` installs the Neovim plugin
- All crate API docs are published to docs.rs and render without warnings
- No breaking changes in any 0.x.y release without corresponding major version bump
- CHANGELOG.md has entries for every release since v0.1.0

### Estimated Timeline

2-3 weeks (after all previous milestones)

---

## Post-1.0 Vision

These items are not committed milestones but represent the long-term direction of the
project. Each is a significant undertaking that may warrant its own major version.

### Custom Font Rendering Engine

For terminals and editors that cannot natively switch fonts per token, build a custom
renderer that composites text from multiple fonts into a single output. This could
work as:
- A terminal multiplexer that intercepts output and renders to a GUI window
- A library that editors can embed to handle multi-font text layout

**Technical sketch:** Use `cosmic-text` or `parley` for shaping, `glyphon` or
`swash` for rasterization, and `wgpu` for GPU-accelerated compositing. The renderer
maintains a font atlas per font family and switches glyphs during layout based on
scope annotations.

**Open questions:** How to handle ligatures that span font boundaries? How to
maintain consistent baseline alignment across fonts with different metrics?

### Variable Font Axis Control Per Token

Go beyond font family switching to control variable font axes (weight, width, slant,
optical size) per token. This enables fine-grained typographic control without
requiring separate font files.

**Technical sketch:** Extend `FontSpec` in `crates/polyfont-core/src/font.rs` with an
`axes: Vec<(String, f32)>` field. The editor integration maps axis values to the
editor's font rendering API (e.g., CSS `font-variation-settings` in VSCode's webview,
or FreeType/ HarfBuzz variable font APIs in native editors).

### Collaborative Font Themes

Allow teams to share polyfont configurations via a git-based theme distribution
system. A team maintains a `polyfont-themes/` directory in their repo; polyfont
discovers and applies it automatically.

**Technical sketch:** Extend `ConfigLoader::find_all_configs()` in
`crates/polyfont-config/src/lib.rs:216-244` to search for shared theme files in
standard locations (e.g., `.polyfont/themes/` in the project root, or a remote URL
specified in a `polyfont.lock` file). Add a `polyfont theme share` command that
creates a GitHub Gist or pushes to a configured theme repository.

### Accessibility Features

- Dyslexia-friendly font recommendations: suggest fonts like OpenDyslexic,
  Atkinson Hyperlegible, or Lexend for users who enable an accessibility mode
- High-contrast font pairings: validate that the chosen fonts have sufficient
  visual distinction at the configured sizes and weights
- Configurable minimum font size and weight overrides
- Screen reader compatibility: ensure font metadata is exposed via accessibility
  APIs where applicable

### LSP-Based Font Suggestion

Use an LSP code action to suggest font assignments. When the user invokes
"polyfont: suggest fonts," the server analyzes the document's scope distribution
and recommends fonts based on typographic best practices (e.g., "use a monospaced
font for keywords, a humanist sans-serif for comments, a geometric sans for
identifiers").

**Technical sketch:** Extend `polyfont-lsp` with a `textDocument/codeAction` handler
that returns font suggestion diagnostics. The suggestions are computed from a
curated database of font pairings indexed by scope and language context.

---

## Timeline Summary

| Milestone | Version | Complexity | Estimated Duration | Dependencies |
|-----------|---------|------------|--------------------|--------------|
| Tree-sitter LSP Integration | v0.2 | L | 4-6 weeks | tree-sitter crates |
| Neovim GUI Native Support | v0.3 | L | 6-10 weeks | Neovim RFC, GUI maintainers |
| Font Management | v0.4 | M | 3-4 weeks | reqwest, platform font crates |
| Performance and Scale | v0.5 | M | 3-4 weeks | criterion, dhat |
| Theme Ecosystem | v0.6 | M | 3-4 weeks | quick-xml |
| Additional Editor Support | v0.7 | L | 6-8 weeks | Editor SDKs |
| Production Release | v1.0 | M | 2-3 weeks | All previous milestones |

**Estimated total to v1.0:** 27-39 weeks (~7-10 months), with v0.2 and v0.3 as the
critical path. v0.4, v0.5, and v0.6 can proceed in parallel once v0.2 lands. v0.7
can begin as soon as v0.2 is stable.

---

## Decision Log

This section records significant architectural decisions with their rationale and
date. Entries are added as decisions are made.

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-20 | Use tower-lsp for LSP server | Mature, async, well-maintained Rust LSP framework |
| 2026-05-20 | TOML for config format | Human-readable, native Rust support, popular in Rust ecosystem |
| 2026-05-20 | TextMate scopes for token classification | Universal syntax highlighting standard, supported by all target editors |
| 2026-05-20 | Per-crate workspace layout | Clean separation of concerns, independent publishable units |
| 2026-05-21 | Feature-gated tree-sitter grammars | Avoids heavy C dependencies for users who don't need parsing |
| 2026-05-21 | std::process::Command for font discovery | No FFI, simple cross-platform shell invocation of fc-list/system_profiler/PowerShell |
| 2026-05-21 | TrieScopeResolver for O(k) scope lookup | Scales to large rule sets without O(n) linear scan |
| 2026-05-21 | Built-in themes with prefix-match fallback | Themes work even with incomplete scope coverage |
| 2026-05-21 | Kitty symbol_map approximation for Helix | Terminal cannot do per-scope fonts; Unicode range mapping is the best available approximation |
