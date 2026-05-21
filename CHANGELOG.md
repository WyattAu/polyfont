# Changelog

All notable changes to the polyfont project are documented here.

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
