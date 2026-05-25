# Migration Guide

Current version: **v0.10.0**

## Overview

polyfont follows [Semantic Versioning](https://semver.org/). The configuration format is stable across versions. CLI flags may gain new options but existing flags remain unchanged.

---

## v0.1 to v0.2

- New `polyfont-parse` crate added to the workspace.
- No user-facing changes.
- LSP now uses tree-sitter for scope resolution when available. This is transparent -- no config changes needed.
- If building from source, add language grammar features to your `[dependencies.polyfont-lsp]` entry:

  ```toml
  polyfont-lsp = { version = "0.2", features = ["rust", "typescript", "python"] }
  ```

---

## v0.2 to v0.3

- Neovim plugin adds two new Lua APIs:

  - `font_metadata_table()` -- returns a table of all resolved font metadata.
  - `get_font_for_scope(scope)` -- returns the resolved font for a given scope string.

- New user command: `:PolyfontMetadata` -- prints font metadata to a scratch buffer.
- Existing plugin usage is unchanged.

---

## v0.3 to v0.4

- New `polyfont-fonts` crate added to the workspace.
- New CLI subcommands:

  - `polyfont font list` -- list discovered fonts on the system.
  - `polyfont font check <name>` -- verify a font is available and print its metadata.

- No configuration format changes.

---

## v0.4 to v0.5

- `TrieScopeResolver` added to `polyfont-scope`, replacing the linear scope matcher. Lookups are now O(k) where k is scope depth.
- Transparent performance improvement. No config or API changes.
- New benchmark suite using `criterion` added at `polyfont-core/benches/`. Run with `cargo bench -p polyfont-core`.

---

## v0.5 to v0.6

- New `polyfont-themes` crate added to the workspace.
- New CLI subcommands:

  - `polyfont theme list` -- list available themes.
  - `polyfont theme show <name>` -- print theme contents.
  - `polyfont theme apply <name>` -- apply a theme to the active config.

- 5 built-in themes: `monaspace`, `serif-mono`, `weight-differentiated`, `minimal`, `maximal`.
- No configuration format changes.

---

## v0.6 to v0.7

- Editor integration documentation added for Zed, Helix, and Sublime Text.
- No code or configuration changes.

---

## v0.7 to v0.8

- CLI `font` and `theme` subcommands fully integrated with config file read/write.
- LSP tree-sitter integration gains `language_id_from_uri()` for automatic language detection from file URIs.
- No breaking changes.

---

## v0.8 to v0.9

- **Variable font axis control** (`polyfont-core`): `NamedAxis` enum (Weight/Width/Slant/OpticalSize/Italic), `AxisValue` (Named or Custom with OpenType 4-char tags). All `FontSpec`/`FontConfig`/`DefaultFontConfig` now carry `axes: Vec<AxisValue>`.
- **Font download system** (`polyfont-fonts`): `FontDownloader`, `FontSource` enum (GitHub/GoogleFonts/Url), known fonts registry, TOML lockfile. Feature-gated behind `download` (reqwest+sha2+zip).
- **LSP font suggestions**: `polyfont/suggestFonts` custom LSP method returning curated font pairings via `FONT_PAIRINGS` database.
- **LSP incremental token caching**: `DocumentState.cached_tokens` avoids redundant re-tokenization on unchanged documents.
- **Collaborative themes** (`polyfont-themes`): `ThemeShare` (gist-style sharing), `ThemeLockfile` (version tracking), `ThemeDiscovery` (project theme scanning).
- **Accessibility checker** (`polyfont-themes`): `AccessibilityChecker` with dyslexia-friendly font recommendations, font-similarity detection.
- **Rendering engine skeleton** (`polyfont-render`): Feature-gated GPU (wgpu+glyphon+cosmic-text) and software (softbuffer+tiny-skia+fontdb) paths.
- **JetBrains plugin skeleton** (`editors/jetbrains/`): IntelliJ plugin with `PolyfontSettings`, `PolyfontApplyAction`, `PolyfontConfigParser` in Kotlin.
- **Neovim font RFC** (`docs/neovim-font-rfc.md`): 441-line RFC proposing `font` field in `nvim_set_hl()`.
- 49 `FontSpec`/`FontConfig` construction sites updated to include `axes` field.
- Workspace expanded from 8 to 9 crates.
- Total test count: 117 (up from 96).

---

## v0.9 to v0.10

- **`font install` CLI subcommand**: Downloads fonts from known sources (6 registries) via `FontDownloader`.
- **`font preview` CLI subcommand**: Generates SVG preview of a font family with custom text.
- **`theme import` CLI subcommand**: Imports VSCode JSON or TextMate `.tmTheme` files into `.polyfont.toml`.
- **`theme export` CLI subcommand**: Exports current config as TOML or VSCode JSON format.
- **Remote theme registry**: `RemoteThemeRegistry` with JSON index, `find()`, `list_names()`. Default includes 3 themes (monaspace-dark, minimal, serif-mono).
- **Zed extension skeleton** (`editors/zed/`): `extension.toml` manifest and `src/lib.rs`.
- **Sublime Text plugin** (`editors/sublime/polyfont.py`): `PolyfontApplyCommand` generates `.sublime-theme` with per-scope font settings.
- **`polyfont check` uses `FontScanner`**: Replaced manual directory scanning with `polyfont-fonts::FontScanner`. Suggests `polyfont font install` for missing fonts.
- `FontMapping` now implements `Default` trait.
- Total tests: 120 (up from 117).
