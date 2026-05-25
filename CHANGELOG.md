# Changelog

All notable changes to the polyfont project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

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
