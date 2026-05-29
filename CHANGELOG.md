# Changelog

All notable changes to the polyfont project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added

- **v0.11: Render engine software path** (`polyfont-render`):
  - `FontCache` with family registration and metric lookup (fontdb-gated).
  - `FontMetrics` with ascent/descent/line_gap/units_per_em and pixel scaling.
  - `BaselineAligner` computes per-family Y offsets for cross-font baseline alignment.
  - `RenderOutput` enum: `Svg(String)` and `Png { width, height, pixels }`.
  - `RenderEngine::render_tokens_svg()` generates SVG with per-token `<text>` elements.
  - `RenderEngine::render_tokens_png()` with built-in 5x8 bitmap font for ASCII (software feature).
  - `PngRenderer` blits scaled bitmap glyphs onto RGBA pixel buffer.
  - `RenderConfig` builder pattern with validation and pixel line height computation.
  - 51 new tests in polyfont-render (up from 4).
- **v0.12: LSP production hardening** (`polyfont-lsp`):
  - Fixed UTF-16 column computation in naive tokenizer (was byte offsets, now proper UTF-16 code units per LSP spec).
  - Client-reported `language_id` from `did_open` now used instead of URI-only inference.
  - `str_len_utf16()` helper for correct non-ASCII position encoding.
  - Debounced `did_change` with 16ms coalesce window (tokio::spawn + abort pattern).
  - `DocumentState` tracks `language_id` for per-document language awareness.
  - `serve_suggest_fonts` now uses `params.language` for filtering.
  - Config load failure sends `window/showMessage` warning to client.
  - 21 new LSP tests (up from 27 to 48), covering UTF-16 edge cases, CJK/emoji, language ID override, build_assignment_entries edge cases.
- **v0.14: Performance benchmarks**:
  - Expanded polyfont-core benchmarks to 1000 rules (was 100).
  - New `trie_resolve` benchmark group comparing TrieScopeResolver vs ScopeMatchEngine.
  - New `polyfont-parse` benchmarks: naive tokenization at 1K/10K/50K lines, byte offset conversion.
  - New `polyfont-config` benchmarks: config parsing at 50/100/200/500 rules, full pipeline measurement.
  - `bench.yml` CI workflow with 10% regression threshold.
- **v0.15: Editor test suites**:
  - **VSCode**: 25 new tests for `buildTextMateRules` and `isBoldWeight` (up from 14 to 39 total).
  - **Sublime**: 29 new Python tests for `_build_font_face`, `_build_theme_entries`, config round-trip.
  - **JetBrains**: 17 JUnit tests for `PolyfontConfigParser` (nested TOML, edge cases).
  - **Neovim**: 37 busted tests for TOML parser, treesitter scope matching, config validation.
- **v0.16: crates.io preparation**:
  - All 9 crates have `description`, `keywords`, `categories`, `readme` metadata.
  - All 9 crates have proper README.md with usage examples.
  - `cargo doc --workspace --no-deps` builds with zero warnings.
  - `criterion` added as workspace dependency.
- **v0.17: Security audit infrastructure**:
  - `SECURITY.md` with reporting process, disclosure timeline, dependency list.
  - `.cargo-deny.toml` with license whitelist, advisory monitoring, source restrictions.
  - Security audit CI job in `ci.yml` (cargo audit + cargo deny check).
  - `cargo audit` reports 0 vulnerabilities across 350 dependencies.
- **Doc comments**: All public API items across polyfont-scope, polyfont-themes, polyfont-parse, polyfont-lsp documented.
- **All clippy suppressions removed** from polyfont-scope (missing_errors_doc, missing_panics_doc).
- **JetBrains**: `untilBuild` widened to 251.* for broader IDE compatibility (2024.2+ / 2025.x).

### Changed

- Total Rust tests: 235 (up from 162).
- Total VSCode tests: 39 (up from 14).
- Total Sublime tests: 29 (new).
- LSP server uses `TextDocumentSyncKind::FULL` with debounced re-tokenization.
- `polyfont-lsp` Cargo.toml: added `tokio` dependency with `time` feature for debounce.
- `polyfont-render` Cargo.toml: `software` feature gates fontdb + tiny-skia + xmlwriter.

## [0.10.1] - 2026-05-25

### Fixed

- **SVG preview width calculation**: Use `text.chars().count()` instead of `text.len()` for correct width with multi-byte UTF-8 characters.
- **`export_vscode` JSON generation**: Replace string formatting with `serde_json` to prevent invalid JSON from special characters in scope names.
- **`FontError::Io` error chain**: Use `#[from] std::io::Error` instead of stringly-typed variant to preserve full error context.
- **`load_merged` unwrap safety**: Replace bare `.unwrap()` with `.expect()` including descriptive message.
- **`FontAtlas` family storage**: Use `HashSet` instead of `Vec` for O(1) deduplication.
- **`AccessibilityChecker` family collection**: Use `HashSet` for O(1) deduplication instead of O(n) `Vec::contains`.
- **`ThemeRegistry::list_themes` caching**: Add `OnceLock` cache to avoid rebuilding theme list on every call.
- **VSCode extension license**: Corrected from MIT to Apache-2.0 to match project license.
- **Rockspec version**: Updated from v0.8.0 to v0.10.0 to match current release.
- **Documentation sidebar**: Removed inconsistent `.md` links from HTML sidebar.
- **Migration guide**: Updated to v0.10.0 with v0.9.0 and v0.10.0 entries.
- **Docs test count**: Updated from 96 to 120.
- **Pre-commit hook**: Fixed `set -e` interaction with result capture pattern.

## [0.10.0] - 2026-05-25

### Added

- **`font install` CLI subcommand**: Downloads fonts from known sources (6 registries) via `FontDownloader`. Wiring to existing download backend with `polyfont-fonts/download` feature.
- **`font preview` CLI subcommand**: Generates SVG preview of a font family with custom text.
- **`theme import` CLI subcommand**: Imports VSCode JSON or TextMate `.tmTheme` files into `.polyfont.toml` with optional font mapping file.
- **`theme export` CLI subcommand**: Exports current config as TOML or VSCode JSON format.
- **Remote theme registry**: `RemoteThemeRegistry` with JSON index, `find()`, `list_names()`, `from_json()`, `to_json()`. Default includes 3 themes (monaspace-dark, minimal, serif-mono).
- **Zed extension skeleton** (`editors/zed/`): `extension.toml` manifest and `src/lib.rs` with `zed::register_extension!`.
- **Sublime Text plugin** (`editors/sublime/polyfont.py`): `PolyfontApplyCommand` reads `.polyfont.toml`, generates `.sublime-theme` with per-scope font settings. `PolyfontGenerateCommand` creates sample config.
- **`polyfont check` uses `FontScanner`**: Replaced manual directory scanning with `polyfont-fonts::FontScanner`. Suggests `polyfont font install` for missing fonts.

### Changed

- Removed old `font_directories()`, `scan_for_font()`, `check_font_available()` functions from CLI (replaced by `FontScanner`).
- CLI now depends on `polyfont-fonts` with `download` feature enabled.
- `FontMapping` now implements `Default` trait.
- Total tests: 120 (up from 117).

## [0.9.0] - 2026-05-25

### Added

- **Variable font axis control** (`polyfont-core`): `NamedAxis` enum (Weight/Width/Slant/OpticalSize/Italic), `AxisValue` (Named or Custom with OpenType 4-char tags), `css_variation_settings()`, `axes_map()`. All `FontSpec`/`FontConfig`/`DefaultFontConfig` now carry `axes: Vec<AxisValue>`.
- **Font download system** (`polyfont-fonts`): `FontDownloader`, `FontSource` enum (GitHub/GoogleFonts/Url), known fonts registry (6 fonts), TOML lockfile. Feature-gated behind `download` (reqwest+sha2+zip).
- **LSP font suggestions**: `polyfont/suggestFonts` custom LSP method returning curated font pairings (16 scope-font recommendations) via `FONT_PAIRINGS` database.
- **LSP incremental token caching**: `DocumentState.cached_tokens` field avoids redundant re-tokenization on unchanged documents.
- **Collaborative themes** (`polyfont-themes`): `ThemeShare` (gist-style sharing), `ThemeLockfile` (version tracking), `ThemeDiscovery` (project theme scanning).
- **Accessibility checker** (`polyfont-themes`): `AccessibilityChecker` with `AccessibilityReport`, dyslexia-friendly font recommendations (OpenDyslexic, Atkinson Hyperlegible, Lexend, Comic Neue, Read Regular), font-similarity detection, 5 accessibility tests.
- **Rendering engine skeleton** (`polyfont-render`): `FontAtlas`, `ScopeAnnotator`, `RenderEngine`, `RenderConfig`, `GlyphPosition`, `RenderedLine`. Feature-gated GPU (`wgpu`+`glyphon`+`cosmic-text`) and software (`softbuffer`+`tiny-skia`+`fontdb`) paths.
- **JetBrains plugin skeleton** (`editors/jetbrains/`): IntelliJ plugin with `PolyfontSettings`, `PolyfontApplyAction`, `PolyfontConfigParser` in Kotlin.
- **Neovim font RFC** (`docs/neovim-font-rfc.md`): 441-line RFC proposing `font` field in `nvim_set_hl()` for neovim/neovim.

### Changed

- 49 `FontSpec`/`FontConfig` construction sites updated across themes/scope crates to include `axes` field.
- Workspace expanded from 8 to 9 crates.

### Tests

- Total test count: 117 (up from 96 in v0.8.0).

## [0.8.0] - 2026-05-21

### Added

- **CONTRIBUTING.md**: Development guide with stability policy, code style, commit conventions, and release process.
- **docs/architecture.md**: Component diagram, data flow, scope matching algorithm, platform support matrix, and extension points.
- **docs/migration.md**: Version-by-version migration guide from v0.1 through v0.8.
- **SemVer CI**: `cargo-semver-checks` job to detect breaking API changes.
- **Cross-compilation release**: CI builds for linux-x64, linux-aarch64, macos-x64, macos-aarch64, windows-x64 with SHA-256 checksums.
- **publish.sh**: Script for crates.io dry-run and publish in dependency order.
- **lua.rocks rockspec**: Neovim plugin metadata for `luarocks install polyfont.nvim`.
- **Stability policy**: No breaking changes to config format, CLI interface, or LSP protocol without major version bump.

## [0.7.0] - 2026-05-21

### Added

- **Editor integration docs**: Setup guides for Zed (`editors/zed/README.md`), Helix (`editors/helix/README.md`), and Sublime Text (`editors/sublime/README.md`).
- Each guide covers installation, configuration generation, and platform-specific limitations.

## [0.6.0] - 2026-05-21

### Added

- **polyfont-themes**: Theme import, export, and registry crate.
  - `ThemeImporter`: Import VSCode and TextMate JSON themes into polyfont config.
  - `ThemeExporter`: Export config as TOML or VSCode-compatible JSON.
  - `ThemeRegistry`: 5 built-in themes (monaspace, serif-mono, weight-differentiated, minimal, maximal).
  - Font mapping with prefix-match fallback and configurable default.
  - 21 unit tests.
- **CLI theme commands**: `polyfont theme list`, `polyfont theme show <name>`, `polyfont theme apply <name>`.

## [0.5.0] - 2026-05-21

### Added

- **TrieScopeResolver**: O(k) scope lookup in polyfont-scope using a prefix trie with specificity-aware matching. 7 new tests.
- **Criterion benchmarks**: Scope matching throughput, engine construction, and specificity sorting across 10-500 rule configurations in `crates/polyfont-core/benches/`.

### Changed

- Scope resolution performance improved from O(n) linear scan to O(k) trie traversal for large rule sets.

## [0.4.0] - 2026-05-21

### Added

- **polyfont-fonts**: Cross-platform font discovery and management crate.
  - `FontDiscovery` trait with platform-specific implementations: `FcListDiscovery` (Linux), `MacOsDiscovery` (macOS), `WindowsDiscovery` (Windows), `FallbackDiscovery`.
  - `FontScanner`: High-level API for availability checks, family listing, and config validation.
  - Font path resolution and family name normalization.
  - 16 unit tests.
- **CLI font commands**: `polyfont font list`, `polyfont font check <family>...`.

## [0.3.0] - 2026-05-21

### Added

- **Neovim font metadata API**: `font_metadata_table()` returns the full font-to-scope mapping for GUI frontend consumption.
- **Neovim `get_font_for_scope()`**: Resolves a single scope string to its assigned font family.
- **`:PolyfontMetadata` command**: Displays current font mapping in a floating window.

## [0.2.0] - 2026-05-21

### Added

- **polyfont-parse**: Tree-sitter-based token parser crate.
  - 10 feature-gated language grammars: rust, c, cpp, go, python, javascript, typescript, java, ruby, html.
  - Naive fallback tokenizer for languages without tree-sitter support.
  - UTF-8/UTF-16 offset conversion, scope extraction from highlight captures.
  - 14 unit tests.
- **LSP tree-sitter integration**: `tokenize_document()` tries tree-sitter first, falls back to naive classifier. `language_id_from_uri()` helper for document language detection.

## [0.1.0] - 2026-05-21

### Added

- **polyfont-core**: Core types (FontRule, FontSpec, TokenInfo, FontAssignment) with ScopeMatchEngine trait. Wildcard `*` scope matching. Display impls for FontWeight and FontStyle. 16 unit tests.
- **polyfont-config**: `.polyfont.toml` parser with validation, hierarchical directory walking, and config merging. 8 unit tests.
- **polyfont-scope**: TextMate scope matching with hierarchical, wildcard, negation, and comma-separated selectors. Trie-based ScopeTree for prefix queries. 14 unit tests.
- **polyfont-cli**: CLI tool with `check`, `vscode`, `neovim`, `kitty`, `dump` subcommands. Font availability checking against system font directories.
- **polyfont-lsp**: LSP server with custom `polyfont/fontAssignments` notification and `polyfont/requestFontAssignments` request handler.
- **VSCode extension**: TypeScript extension using VSCode 1.97+ `textMateRules` fontFamily API (PR #263403). TOML config parser, file watcher with 500ms debounce.
- **Neovim plugin**: Lua plugin with Tree-sitter integration, decoration providers, font metadata for GUI consumption. Commands: `:PolyfontSetup`, `:PolyfontReload`, `:PolyfontStatus`, `:PolyfontClear`.
- **CI/CD**: GitHub Actions workflow with check/test/build jobs across Linux, macOS, Windows.
- **Example config**: `polyfont.example.toml` with 17 font rules covering common scopes.
- **Pre-commit hooks**: Format, clippy, and test validation before commit and push.
- **Documentation**: README with configuration reference, scope reference, editor compatibility matrix.

### Technical Decisions

- VSCode integration uses `editor.tokenColorCustomizations.textMateRules` with `fontFamily` support (merged in VSCode PR #263403, December 2025, available in VSCode 1.97+).
- Neovim integration uses highlight groups with extmarks and stores font metadata in `vim.g.polyfont_font_map` for GUI consumption.
- Scope matching uses most-specific-wins strategy with specificity scoring by dot-separated segment count.
- Configuration format is TOML for human readability, comment support, and ease of parsing.
- Monorepo structure: Cargo workspace for Rust crates, separate directories for editor extensions.

### Known Limitations

- LSP tokenizer uses line-prefix heuristics rather than Tree-sitter parsing. Inline tokens (e.g., comments after code on the same line) are not detected.
- Neovim GUI support for per-token font families requires GUI-specific implementation. Goneovim has partial support; Neovide requires patching.
- Kitty terminal integration generates static `symbol_map` configuration rather than runtime font switching.
- Windows font scanning may miss fonts installed in user-specific directories not on the standard search paths.
